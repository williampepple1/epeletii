//! Game room management — manages multiple concurrent games.
//! Uses mpsc channels for broadcasting messages to connected players.

use crate::dictionary::Dictionary;
use crate::game::Game;
use crate::protocol::PlayerInfo;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

/// A game room with connected players.
pub struct Room {
    pub id: String,
    pub name: String,
    pub game: Game,
    /// Map of player_id -> message sender channel
    pub senders: HashMap<String, mpsc::UnboundedSender<String>>,
    pub max_players: usize,
}

impl Room {
    pub fn new(name: String, dictionary: Dictionary) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            name,
            game: Game::new(dictionary),
            senders: HashMap::new(),
            max_players: 4,
        }
    }

    /// Register a player's message sender channel.
    pub fn register_sender(&mut self, player_id: &str, sender: mpsc::UnboundedSender<String>) {
        self.senders.insert(player_id.to_string(), sender);
    }

    /// Send a JSON message to a specific player.
    pub fn send_to(&self, player_id: &str, msg: &impl serde::Serialize) {
        if let Some(sender) = self.senders.get(player_id) {
            if let Ok(json) = serde_json::to_string(msg) {
                let _ = sender.send(json);
            }
        }
    }

    /// Broadcast a JSON message to all connected players in the room.
    pub fn broadcast(&self, msg: &impl serde::Serialize) {
        if let Ok(json) = serde_json::to_string(msg) {
            for sender in self.senders.values() {
                let _ = sender.send(json.clone());
            }
        }
    }

    /// Broadcast to all players except one.
    pub fn broadcast_except(&self, exclude: &str, msg: &impl serde::Serialize) {
        if let Ok(json) = serde_json::to_string(msg) {
            for (pid, sender) in &self.senders {
                if pid != exclude && sender.send(json.clone()).is_err() {
                    // stale sender — cleaned up on disconnect
                }
            }
        }
    }

    /// Remove a player's sender (on disconnect).
    pub fn remove_sender(&mut self, player_id: &str) {
        self.senders.remove(player_id);
    }

    /// Check if all players have active senders.
    pub fn all_connected(&self) -> bool {
        self.game.players.iter().all(|p| self.senders.contains_key(&p.id))
    }

    pub fn player_info_list(&self) -> Vec<PlayerInfo> {
        self.game
            .players
            .iter()
            .map(|p| PlayerInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                score: p.score,
                tile_count: p.rack.len(),
            })
            .collect()
    }

    /// Convert board to serializable format.
    pub fn board_squares(&self) -> Vec<Vec<crate::protocol::BoardSquare>> {
        self.game
            .board
            .squares
            .iter()
            .map(|row| {
                row.iter()
                    .map(|sq| crate::protocol::BoardSquare {
                        row: sq.row,
                        col: sq.col,
                        premium: format!("{:?}", sq.premium),
                        tile: sq.tile.clone(),
                        owner: sq.owner,
                    })
                    .collect()
            })
            .collect()
    }
}

/// Manages all game rooms.
pub struct RoomManager {
    rooms: HashMap<String, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self, name: String, dictionary: Dictionary) -> &mut Room {
        let room = Room::new(name, dictionary);
        let id = room.id.clone();
        self.rooms.insert(id.clone(), room);
        self.rooms.get_mut(&id).unwrap()
    }

    pub fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn get_room_mut(&mut self, id: &str) -> Option<&mut Room> {
        self.rooms.get_mut(id)
    }

    pub fn remove_room(&mut self, id: &str) {
        self.rooms.remove(id);
    }
}
