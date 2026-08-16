//! Epeletii — Multiplayer Ibani Scrabble game server.
//!
//! WebSocket-based game server. Each connection gets an mpsc channel
//! so the server can broadcast messages to all players in a room.

#![allow(dead_code)]

mod auth;
mod board;
mod dictionary;
mod email;
mod game;
mod persistence;
mod protocol;
mod room;
mod tiles;

use crate::auth::AuthService;
use crate::dictionary::Dictionary;
use crate::persistence::{GameStore, snapshot_to_game};
use crate::protocol::{ClientMessage, ServerMessage};
use crate::room::RoomManager;

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

struct AppState {
    rooms: Mutex<RoomManager>,
    auth: AuthService,
    game_store: GameStore,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr = "0.0.0.0:9001";
    log::info!("Starting Epeletii server on {}", addr);

    let mongo_uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017/epeletii".to_string());
    let auth = AuthService::new(&mongo_uri, "epeletii")
        .await
        .expect("Failed to connect to MongoDB");
    log::info!("Connected to MongoDB at {}", mongo_uri);

    let game_store = GameStore::new(&mongo_uri, "epeletii")
        .await
        .expect("Failed to connect GameStore to MongoDB");

    // Restore all active games from MongoDB on startup
    let room_manager = {
        let mut rm = RoomManager::new();
        let snapshots = game_store.load_all_active_games().await;
        log::info!("Restoring {} active game(s) from MongoDB", snapshots.len());
        for snap in snapshots {
            let game = snapshot_to_game(&snap, Dictionary::load());
            rm.restore_room(snap.room_id.clone(), game);
            log::info!("Restored room {}", snap.room_id);
        }
        rm
    };

    let state = Arc::new(AppState {
        rooms: Mutex::new(room_manager),
        auth,
        game_store,
    });

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    while let Ok((stream, peer)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(handle_connection(stream, peer, state));
    }
}

async fn handle_connection(stream: TcpStream, peer: SocketAddr, state: Arc<AppState>) {
    log::info!("New connection from {}", peer);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    let (mut ws_sender, ws_receiver) = ws_stream.split();

    // Channel for server -> client messages
    let (tx_base, mut rx) = mpsc::unbounded_channel::<String>();

    // Writer task: forwards mpsc messages to WebSocket
    tokio::spawn(async move {
        while let Some(json) = rx.recv().await {
            if ws_sender
                .send(Message::Text(json.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let tx = tx_base.clone();
    let mut my_player_id: Option<String> = None;
    let mut my_room_id: Option<String> = None;

    // Helper to send an error to this connection
    let send_err = |msg: &str| {
        let _ = tx_base
            .send(
                serde_json::to_string(&ServerMessage::Error {
                    message: msg.to_string(),
                })
                .unwrap(),
            );
    };

    // Main read loop
    let mut recv = ws_receiver;
    while let Some(msg) = recv.next().await {
        let raw = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(_)) => continue,
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(e) => {
                send_err(&format!("Invalid message: {}", e));
                continue;
            }
        };

        match client_msg {
            ClientMessage::SignUp { email, password, display_name } => {
                match state.auth.signup(&email, &password, &display_name).await {
                    Ok((token, user)) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthSuccess {
                            token,
                            email: user.email,
                            display_name: user.display_name,
                        }).unwrap());
                    }
                    Err(e) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthError {
                            message: e,
                        }).unwrap());
                    }
                }
            }

            ClientMessage::SignIn { email, password } => {
                match state.auth.signin(&email, &password).await {
                    Ok((token, user)) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthSuccess {
                            token,
                            email: user.email,
                            display_name: user.display_name,
                        }).unwrap());
                    }
                    Err(e) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthError {
                            message: e,
                        }).unwrap());
                    }
                }
            }

            ClientMessage::ForgotPassword { email } => {
                match state.auth.request_password_reset(&email).await {
                    Ok(_) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::ForgotPasswordSent).unwrap());
                    }
                    Err(e) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthError {
                            message: e,
                        }).unwrap());
                    }
                }
            }

            ClientMessage::ResetPassword { email, token, new_password } => {
                match state.auth.reset_password(&email, &token, &new_password).await {
                    Ok(_) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::ResetPasswordSuccess).unwrap());
                    }
                    Err(e) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::AuthError {
                            message: e,
                        }).unwrap());
                    }
                }
            }

            ClientMessage::CreateRoom { player_name, token } => {
                let mut rooms = state.rooms.lock().await;
                let room = rooms.create_room(
                    format!("{}'s Game", player_name),
                    Dictionary::load(),
                );
                let mut new_id = uuid::Uuid::new_v4().to_string();
                if let Some(ref t) = token {
                    if let Ok(claims) = state.auth.verify_token(t) {
                        new_id = claims.sub;
                    }
                }
                log::info!("Room {} created by {} (player {}) [from {}]", room.id, player_name, new_id, peer);
                room.game.add_player(new_id.clone(), player_name);
                room.register_sender(&new_id, tx.clone());

                room.send_to(
                    &new_id,
                    &ServerMessage::RoomCreated {
                        room_id: room.id.clone(),
                        player_id: new_id.clone(),
                        players: room.player_info_list(),
                    },
                );

                my_player_id = Some(new_id);
                my_room_id = Some(room.id.clone());
            }

            ClientMessage::JoinRoom { room_id, player_name, token } => {
                let mut rooms = state.rooms.lock().await;
                if let Some(room) = rooms.get_room_mut(&room_id) {
                    if room.game.players.len() >= room.max_players {
                        send_err("Room is full");
                        continue;
                    }
                    let mut new_id = uuid::Uuid::new_v4().to_string();
                    if let Some(ref t) = token {
                        if let Ok(claims) = state.auth.verify_token(t) {
                            new_id = claims.sub;
                        }
                    }
                    log::info!("Player {} ({}) joining room {} [from {}]", player_name, new_id, room_id, peer);
                    room.game.add_player(new_id.clone(), player_name);
                    room.register_sender(&new_id, tx.clone());

                    let players = room.player_info_list();
                    room.send_to(
                        &new_id,
                        &ServerMessage::RoomJoined {
                            room_id: room_id.clone(),
                            player_id: new_id.clone(),
                            players,
                        },
                    );

                    // Notify existing players
                    if let Some(last) = room.player_info_list().last() {
                        room.broadcast_except(&new_id, &ServerMessage::PlayerJoined {
                            player: last.clone(),
                        });
                    }

                    my_player_id = Some(new_id);
                    my_room_id = Some(room_id);
                } else {
                    send_err(&format!("Room {} not found", room_id));
                }
            }

            ClientMessage::Ready => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let mut rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room_mut(rid) {
                        let player_count_before = room.game.players.len();
                        let ready_count_before = room.game.players.iter().filter(|p| p.ready).count();

                        if let Some(p) = room.game.players.iter_mut().find(|p| p.id == *pid) {
                            p.ready = true;
                            log::info!("Player {} ({}) ready in room {} [from {}]", p.name, pid, rid, peer);
                        } else {
                            log::warn!("Player {} not found in room {} [from {}] (players: {:?})", pid, rid, peer,
                                room.game.players.iter().map(|p| p.id.as_str()).collect::<Vec<_>>());
                        }

                        let ready_count_after = room.game.players.iter().filter(|p| p.ready).count();
                        log::info!("Room {} ready status: {}/{} players ready (was {}/{})",
                            rid, ready_count_after, player_count_before, ready_count_before, player_count_before);

                        if room.game.all_ready() {
                            log::info!("All players ready! Draw for first in room {}", rid);

                            // Draw to determine first player
                            let (winner_idx, draws) = room.game.draw_for_first();

                            // Build draw entries with player names
                            let draw_entries: Vec<crate::protocol::DrawEntry> = draws
                                .iter()
                                .map(|(pid, letter)| {
                                    let name = room.game.players
                                        .iter()
                                        .find(|p| p.id == *pid)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    crate::protocol::DrawEntry {
                                        player_id: pid.clone(),
                                        player_name: name,
                                        letter: letter.clone(),
                                    }
                                })
                                .collect();

                            let first_player_id = room.game.players[winner_idx].id.clone();
                            room.broadcast(&ServerMessage::DrawResult {
                                draws: draw_entries,
                                first_player_id: first_player_id.clone(),
                            });

                            // Start the game with the winner going first
                            if let Err(e) = room.game.start(winner_idx) {
                                room.send_to(pid, &ServerMessage::Error {
                                    message: format!("Failed to start: {}", e),
                                });
                            } else {
                                let board = room.board_squares();
                                let players = room.player_info_list();
                                room.broadcast(&ServerMessage::GameStarted {
                                    board,
                                    players: players.clone(),
                                    current_turn: room.game.current_player_index() as u8,
                                });
                                for p in &room.game.players {
                                    let tiles: Vec<String> =
                                        p.rack.iter().map(|t| t.letter.clone()).collect();
                                    room.send_to(&p.id, &ServerMessage::YourTiles { tiles });
                                }
                                // Tell the first player it's their turn
                                let first_idx = room.game.current_player_index();
                                room.send_to(
                                    &room.game.players[first_idx].id,
                                    &ServerMessage::YourTurn,
                                );
                                log::info!("Game started in room {}", rid);
                                // Persist initial game state
                                state.game_store.save_game(rid, &room.game).await;
                            }
                        }
                    }
                }
            }

            ClientMessage::PlaceTiles { placements } => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let mut rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room_mut(rid) {
                        // SECURITY: Only the current player can place tiles
                        let current = room.game.current_player_index();
                        if room.game.players[current].id != *pid {
                            room.send_to(pid, &ServerMessage::Error {
                                message: "It's not your turn".to_string(),
                            });
                            continue;
                        }
                        let tuples: Vec<(usize, usize, String)> = placements
                            .iter()
                            .map(|p| (p.row, p.col, p.letter.clone()))
                            .collect();

                        match room.game.place_tiles(&tuples) {
                            Ok((words, score)) => {
                                // If the play used 10 tiles, broadcast a BINGO chat message
                                if tuples.len() == 10 {
                                    let player_name = room
                                        .game
                                        .players
                                        .iter()
                                        .find(|p| p.id == *pid)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_else(|| "Someone".to_string());
                                    room.broadcast(&ServerMessage::Chat {
                                        player_id: "system".to_string(),
                                        player_name: "System".to_string(),
                                        message: format!("🎉 BINGO! {} played all 10 tiles and gets a +50 point bonus!", player_name),
                                    });
                                }
                                room.broadcast(&ServerMessage::MoveMade {
                                    player_id: pid.clone(),
                                    placements,
                                    score,
                                    words_formed: words,
                                });
                                if room.game.check_game_end() {
                                    let winner = room.game.get_winner();
                                    let scores: Vec<u32> =
                                        room.game.players.iter().map(|p| p.score).collect();

                                    // Update database for all registered players
                                    for p in &room.game.players {
                                        if p.id.contains('@') {
                                            let won = Some(p.id.clone()) == winner;
                                            state.auth.record_game_result(&p.id, p.score, won).await;
                                        }
                                    }

                                    // Remove persisted game — it's over
                                    state.game_store.delete_game(rid).await;

                                    room.broadcast(&ServerMessage::GameOver {
                                        winner,
                                        final_scores: scores,
                                        reason: "Player used all tiles (final scores adjusted for remaining tiles)".to_string(),
                                    });
                                } else {
                                    room.game.next_turn();
                                    // Persist after every successful move
                                    state.game_store.save_game(rid, &room.game).await;
                                    let scores: Vec<u32> =
                                        room.game.players.iter().map(|p| p.score).collect();
                                    let board = room.board_squares();
                                    room.broadcast(&ServerMessage::BoardUpdate {
                                        board,
                                        scores,
                                        current_turn: room.game.current_player_index() as u8,
                                        tiles_remaining: room.game.tiles_remaining(),
                                    });
                                    let next = room.game.current_player_index();
                                    let ntiles: Vec<String> = room.game.players[next]
                                        .rack
                                        .iter()
                                        .map(|t| t.letter.clone())
                                        .collect();
                                    room.send_to(
                                        &room.game.players[next].id,
                                        &ServerMessage::YourTiles { tiles: ntiles },
                                    );
                                    room.send_to(
                                        &room.game.players[next].id,
                                        &ServerMessage::YourTurn,
                                    );
                                }
                            }
                            Err(e) => {
                                // Forfeit turn on invalid word (Scrabble rule)
                                let idx = room.game.current_player_index();
                                let restored_tiles: Vec<String> = room.game.players[idx]
                                    .rack
                                    .iter()
                                    .map(|t| t.letter.clone())
                                    .collect();
                                room.send_to(pid, &ServerMessage::YourTiles { tiles: restored_tiles });
                                room.send_to(pid, &ServerMessage::Error {
                                    message: format!("Invalid move: {}", e),
                                });

                                // Advance turn to next player
                                room.game.next_turn();
                                let scores: Vec<u32> =
                                    room.game.players.iter().map(|p| p.score).collect();
                                let board = room.board_squares();
                                room.broadcast(&ServerMessage::BoardUpdate {
                                    board,
                                    scores,
                                    current_turn: room.game.current_player_index() as u8,
                                    tiles_remaining: room.game.tiles_remaining(),
                                });
                                let next = room.game.current_player_index();
                                let ntiles: Vec<String> = room.game.players[next]
                                    .rack
                                    .iter()
                                    .map(|t| t.letter.clone())
                                    .collect();
                                room.send_to(
                                    &room.game.players[next].id,
                                    &ServerMessage::YourTiles { tiles: ntiles },
                                );
                                room.send_to(
                                    &room.game.players[next].id,
                                    &ServerMessage::YourTurn,
                                );
                            }
                        }
                    }
                }
            }

            ClientMessage::PassTurn => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let mut rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room_mut(rid) {
                        let current = room.game.current_player_index();
                        if room.game.players[current].id != *pid {
                            room.send_to(pid, &ServerMessage::Error {
                                message: "It's not your turn".to_string(),
                            });
                            continue;
                        }
                        room.broadcast(&ServerMessage::PlayerPassed {
                            player_id: pid.clone(),
                        });
                        room.game.pass_turn();
                        if matches!(room.game.phase, crate::game::GamePhase::Finished) {
                            let winner = room.game.get_winner();
                            let scores: Vec<u32> =
                                room.game.players.iter().map(|p| p.score).collect();

                            // Update database for all registered players
                            for p in &room.game.players {
                                if p.id.contains('@') {
                                    let won = Some(p.id.clone()) == winner;
                                    state.auth.record_game_result(&p.id, p.score, won).await;
                                }
                            }

                            // Remove persisted game — it's over
                            state.game_store.delete_game(rid).await;

                            room.broadcast(&ServerMessage::GameOver {
                                winner,
                                final_scores: scores,
                                reason: "All players passed consecutively".to_string(),
                            });
                        } else {
                            // Persist after pass
                            state.game_store.save_game(rid, &room.game).await;
                            let next = room.game.current_player_index();
                            let ntiles: Vec<String> = room.game.players[next]
                                .rack
                                .iter()
                                .map(|t| t.letter.clone())
                                .collect();
                            room.send_to(
                                &room.game.players[next].id,
                                &ServerMessage::YourTiles { tiles: ntiles },
                            );
                            room.send_to(
                                &room.game.players[next].id,
                                &ServerMessage::YourTurn,
                            );
                        }
                    }
                }
            }

            ClientMessage::ExchangeTiles { letters } => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let mut rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room_mut(rid) {
                        let current = room.game.current_player_index();
                        if room.game.players[current].id != *pid {
                            room.send_to(pid, &ServerMessage::Error {
                                message: "It's not your turn".to_string(),
                            });
                            continue;
                        }
                        match room.game.exchange_tiles(&letters) {
                            Ok(()) => {
                                let idx = room.game.current_player_index();
                                let tiles: Vec<String> = room.game.players[idx]
                                    .rack
                                    .iter()
                                    .map(|t| t.letter.clone())
                                    .collect();
                                room.send_to(pid, &ServerMessage::YourTiles { tiles });
                                room.broadcast(&ServerMessage::TilesExchanged {
                                    player_id: pid.clone(),
                                    count: letters.len(),
                                });

                                // Advance turn to next player
                                room.game.next_turn();
                                // Persist after tile exchange
                                state.game_store.save_game(rid, &room.game).await;

                                let scores: Vec<u32> =
                                    room.game.players.iter().map(|p| p.score).collect();
                                let board = room.board_squares();
                                room.broadcast(&ServerMessage::BoardUpdate {
                                    board,
                                    scores,
                                    current_turn: room.game.current_player_index() as u8,
                                    tiles_remaining: room.game.tiles_remaining(),
                                });

                                let next = room.game.current_player_index();
                                let ntiles: Vec<String> = room.game.players[next]
                                    .rack
                                    .iter()
                                    .map(|t| t.letter.clone())
                                    .collect();
                                room.send_to(
                                    &room.game.players[next].id,
                                    &ServerMessage::YourTiles { tiles: ntiles },
                                );
                                room.send_to(
                                    &room.game.players[next].id,
                                    &ServerMessage::YourTurn,
                                );
                            }
                            Err(e) => {
                                room.send_to(pid, &ServerMessage::Error {
                                    message: format!("Exchange failed: {}", e),
                                });
                            }
                        }
                    }
                }
            }

            ClientMessage::Chat { message } => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room(rid) {
                        let name = room
                            .game
                            .players
                            .iter()
                            .find(|p| p.id == *pid)
                            .map(|p| p.name.clone())
                            .unwrap_or_default();
                        room.broadcast(&ServerMessage::Chat {
                            player_id: pid.clone(),
                            player_name: name,
                            message,
                        });
                    }
                }
            }

            ClientMessage::Resign => {
                if let (Some(ref pid), Some(ref rid)) = (&my_player_id, &my_room_id) {
                    let mut rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room_mut(rid) {
                        let resigning_player_idx = room.game.players.iter().position(|p| p.id == *pid);
                        if let Some(idx) = resigning_player_idx {
                            let resigning_score = room.game.players[idx].score;
                            let resigning_name = room.game.players[idx].name.clone();
                            
                            // Set resigning player's score to 0
                            room.game.players[idx].score = 0;

                            // Add their score to opponent(s)
                            let opponent_count = room.game.players.len() - 1;
                            let mut winner_label = "Opponent".to_string();
                            if opponent_count > 0 {
                                let share = resigning_score / opponent_count as u32;
                                for i in 0..room.game.players.len() {
                                    if i != idx {
                                        room.game.players[i].score += share;
                                    }
                                }
                            }

                            // The winner is the player with the highest score (not the resigning one)
                            let winner = room.game.players
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| *i != idx)
                                .max_by_key(|(_, p)| p.score)
                                .map(|(_, p)| p.id.clone());

                            if let Some(ref wid) = winner {
                                if let Some(p) = room.game.players.iter().find(|p| p.id == *wid) {
                                    winner_label = p.name.clone();
                                }
                            }

                            room.game.end_game(winner.clone(), format!("{} resigned", resigning_name));

                            // Update database for all registered players
                            for p in &room.game.players {
                                if p.id.contains('@') {
                                    let won = Some(p.id.clone()) == winner;
                                    state.auth.record_game_result(&p.id, p.score, won).await;
                                }
                            }

                            // Remove persisted game — it's over
                            state.game_store.delete_game(rid).await;

                            let scores: Vec<u32> =
                                room.game.players.iter().map(|p| p.score).collect();
                            
                            room.broadcast(&ServerMessage::GameOver {
                                winner,
                                final_scores: scores,
                                reason: format!("{} resigned. Their points were transferred to {}!", resigning_name, winner_label),
                            });
                        }
                    }
                }
            }

            ClientMessage::LookupWord { word } => {
                if let Some(ref rid) = my_room_id {
                    let rooms = state.rooms.lock().await;
                    if let Some(room) = rooms.get_room(rid) {
                        let definitions = room.game.dictionary.lookup(&word).unwrap_or_default();
                        let _ = tx.send(serde_json::to_string(&ServerMessage::WordDefinition {
                            word,
                            definitions,
                        }).unwrap());
                    }
                } else {
                    let dict = Dictionary::load();
                    let definitions = dict.lookup(&word).unwrap_or_default();
                    let _ = tx.send(serde_json::to_string(&ServerMessage::WordDefinition {
                        word,
                        definitions,
                    }).unwrap());
                }
            }

            ClientMessage::RejoinRoom { room_id, player_id } => {
                // First check if room is in memory; if not, try to restore from MongoDB
                {
                    let rooms = state.rooms.lock().await;
                    if rooms.get_room(&room_id).is_none() {
                        drop(rooms);
                        // Try to load from MongoDB
                        if let Some(snap) = state.game_store.load_game(&room_id).await {
                            let game = snapshot_to_game(&snap, Dictionary::load());
                            let mut rooms = state.rooms.lock().await;
                            rooms.restore_room(room_id.clone(), game);
                            log::info!("Restored room {} from MongoDB for rejoin", room_id);
                        }
                    }
                }

                let mut rooms = state.rooms.lock().await;
                if let Some(room) = rooms.get_room_mut(&room_id) {
                    if room.game.players.iter().any(|p| p.id == player_id) {
                        log::info!("Player {} rejoining room {} [from {}]", player_id, room_id, peer);
                        room.register_sender(&player_id, tx.clone());

                        my_player_id = Some(player_id.clone());
                        my_room_id = Some(room_id.clone());

                        // Send the current RoomState to the rejoining player
                        let board = room.board_squares();
                        let players = room.player_info_list();
                        let scores = room.game.players.iter().map(|p| p.score).collect();
                        let current_turn = room.game.current_player_index() as u8;
                        let tiles_remaining = room.game.tiles_remaining();
                        let game_started = matches!(room.game.phase, crate::game::GamePhase::Playing | crate::game::GamePhase::Finished);
                        let game_over = matches!(room.game.phase, crate::game::GamePhase::Finished);
                        let winner = room.game.winner.clone();

                        let _ = tx.send(serde_json::to_string(&ServerMessage::RoomState {
                            room_id: room_id.clone(),
                            player_id: Some(player_id.clone()),
                            players,
                            board,
                            scores,
                            current_turn,
                            tiles_remaining,
                            game_started,
                            game_over,
                            winner,
                        }).unwrap());

                        // Also send their current tiles (rack)
                        if let Some(player) = room.game.players.iter().find(|p| p.id == player_id) {
                            let tiles = player.rack.iter().map(|t| t.letter.clone()).collect();
                            let _ = tx.send(serde_json::to_string(&ServerMessage::YourTiles { tiles }).unwrap());
                        }

                        // If it is currently their turn, notify them!
                        if room.game.players[room.game.current_player_index()].id == player_id {
                            let _ = tx.send(serde_json::to_string(&ServerMessage::YourTurn).unwrap());
                        }
                    } else {
                        send_err("Player not found in room");
                    }
                } else {
                    send_err("Room not found");
                }
            }

            ClientMessage::GetLeaderboard => {
                match state.auth.get_leaderboard().await {
                    Ok(entries) => {
                        let _ = tx.send(serde_json::to_string(&ServerMessage::Leaderboard {
                            entries,
                        }).unwrap());
                    }
                    Err(e) => {
                        send_err(&format!("Failed to load leaderboard: {}", e));
                    }
                }
            }

            ClientMessage::GetActiveRooms => {
                let rooms = state.rooms.lock().await;
                let active_rooms: Vec<crate::protocol::ActiveRoomInfo> = rooms
                    .get_all_rooms()
                    .iter()
                    .filter(|r| r.game.phase != crate::game::GamePhase::Finished)
                    .map(|r| crate::protocol::ActiveRoomInfo {
                        id: r.id.clone(),
                        name: r.name.clone(),
                        player_count: r.game.players.len(),
                        max_players: r.max_players,
                        game_started: r.game.phase == crate::game::GamePhase::Playing,
                    })
                    .collect();

                let _ = tx.send(serde_json::to_string(&ServerMessage::ActiveRooms {
                    rooms: active_rooms,
                }).unwrap());
            }

            ClientMessage::SpectateRoom { room_id } => {
                let mut rooms = state.rooms.lock().await;
                if let Some(room) = rooms.get_room_mut(&room_id) {
                    let new_id = format!("spectator_{}", uuid::Uuid::new_v4());
                    log::info!("Spectator ({}) spectating room {} [from {}]", new_id, room_id, peer);
                    room.register_sender(&new_id, tx.clone());

                    let game_started = matches!(room.game.phase, crate::game::GamePhase::Playing | crate::game::GamePhase::Finished);
                    let game_over = matches!(room.game.phase, crate::game::GamePhase::Finished);

                    let board = room.board_squares();
                    let players = room.player_info_list();
                    let scores = room.game.players.iter().map(|p| p.score).collect();
                    let current_turn = room.game.current_player_index() as u8;
                    let tiles_remaining = room.game.tiles_remaining();
                    let winner = room.game.winner.clone();

                    let _ = tx.send(serde_json::to_string(&ServerMessage::RoomState {
                        room_id: room_id.clone(),
                        player_id: Some(new_id.clone()),
                        players,
                        board,
                        scores,
                        current_turn,
                        tiles_remaining,
                        game_started,
                        game_over,
                        winner,
                    }).unwrap());

                    my_player_id = Some(new_id);
                    my_room_id = Some(room_id);
                } else {
                    send_err(&format!("Room {} not found", room_id));
                }
            }
        }
    }

    // Cleanup on disconnect
    if let Some(pid) = my_player_id {
        if let Some(rid) = my_room_id {
            let mut rooms = state.rooms.lock().await;
            if let Some(room) = rooms.get_room_mut(&rid) {
                room.remove_sender(&pid);
                if room.senders.is_empty() {
                    // Do not remove room immediately to allow reconnects
                    // rooms.remove_room(&rid);
                    // log::info!("Removed empty room {}", rid);
                }
            }
        }
    }

    log::info!("Client disconnected: {}", peer);
}
