//! Epeletii — Multiplayer Ibani Scrabble game server.
//!
//! WebSocket-based game server. Each connection gets an mpsc channel
//! so the server can broadcast messages to all players in a room.

#![allow(dead_code)]

mod auth;
mod board;
mod dictionary;
mod game;
mod protocol;
mod room;
mod tiles;

use crate::auth::AuthService;
use crate::dictionary::Dictionary;
use crate::game::GamePhase;
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
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr = "0.0.0.0:9001";
    log::info!("Starting Epeletii server on {}", addr);

    let mongo_uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let auth = AuthService::new(&mongo_uri, "epeletii")
        .await
        .expect("Failed to connect to MongoDB");
    log::info!("Connected to MongoDB at {}", mongo_uri);

    let state = Arc::new(AppState {
        rooms: Mutex::new(RoomManager::new()),
        auth,
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

            ClientMessage::CreateRoom { player_name } => {
                let mut rooms = state.rooms.lock().await;
                let room = rooms.create_room(
                    format!("{}'s Game", player_name),
                    Dictionary::load(),
                );
                let new_id = uuid::Uuid::new_v4().to_string();
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

            ClientMessage::JoinRoom { room_id, player_name } => {
                let mut rooms = state.rooms.lock().await;
                if let Some(room) = rooms.get_room_mut(&room_id) {
                    if room.game.players.len() >= room.max_players {
                        send_err("Room is full");
                        continue;
                    }
                    let new_id = uuid::Uuid::new_v4().to_string();
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
                                log::info!("Game started in room {}", rid);
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
                                    room.broadcast(&ServerMessage::GameOver {
                                        winner,
                                        final_scores: scores,
                                        reason: "Player used all tiles".to_string(),
                                    });
                                } else {
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
                        if matches!(room.game.phase, GamePhase::Finished) {
                            let winner = room.game.get_winner();
                            let scores: Vec<u32> =
                                room.game.players.iter().map(|p| p.score).collect();
                            room.broadcast(&ServerMessage::GameOver {
                                winner,
                                final_scores: scores,
                                reason: "All players passed consecutively".to_string(),
                            });
                        } else {
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
                        room.game
                            .end_game(None, format!("Player {} resigned", pid));
                        let scores: Vec<u32> =
                            room.game.players.iter().map(|p| p.score).collect();
                        room.broadcast(&ServerMessage::GameOver {
                            winner: None,
                            final_scores: scores,
                            reason: "A player resigned".to_string(),
                        });
                    }
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
                    rooms.remove_room(&rid);
                    log::info!("Removed empty room {}", rid);
                }
            }
        }
    }

    log::info!("Client disconnected: {}", peer);
}
