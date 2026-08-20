//! Game persistence — save and restore game state to/from MongoDB.

use crate::board::{Board, Premium, Square};
use crate::game::{Game, GamePhase, Player};
use crate::dictionary::Dictionary;
use crate::tiles::Tile;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

/// A serializable snapshot of the full game state saved to MongoDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// The room ID (used as the MongoDB document key).
    pub room_id: String,
    /// Serialized board state.
    pub board: Vec<Vec<SquareSnapshot>>,
    /// All players with their racks.
    pub players: Vec<PlayerSnapshot>,
    /// Remaining tiles in the bag.
    pub tile_bag: Vec<TileSnapshot>,
    /// Index into players[] of whose turn it is.
    pub current_turn: usize,
    /// Current game phase.
    pub phase: String,
    /// Number of consecutive passes.
    pub consecutive_passes: u32,
    /// Winner player ID if game is finished.
    pub winner: Option<String>,
    /// Timestamp of last update (Unix seconds, for TTL).
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquareSnapshot {
    pub row: usize,
    pub col: usize,
    pub premium: String,
    pub tile: Option<String>,
    pub owner: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub rack: Vec<TileSnapshot>,
    pub ready: bool,
    #[serde(default)]
    pub is_bot: bool,
    #[serde(default)]
    pub bot_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSnapshot {
    pub letter: String,
    pub value: u8,
}

/// Wraps the MongoDB `games` collection for game persistence.
pub struct GameStore {
    collection: Collection<GameSnapshot>,
}

impl GameStore {
    pub async fn new(mongo_uri: &str, db_name: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(mongo_uri).await?;
        let db = client.database(db_name);
        let collection = db.collection::<GameSnapshot>("games");

        // Create TTL index: auto-delete documents 24h after last update
        let index = mongodb::IndexModel::builder()
            .keys(doc! { "updated_at": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(24 * 60 * 60))
                    .build(),
            )
            .build();
        let _ = collection.create_index(index).await;

        Ok(Self { collection })
    }

    /// Save (upsert) the current game state to MongoDB.
    pub async fn save_game(&self, room_id: &str, game: &Game) {
        let snapshot = game_to_snapshot(room_id, game);
        let filter = doc! { "room_id": room_id };
        let opts = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();
        if let Err(e) = self.collection.replace_one(filter, snapshot).with_options(opts).await {
            log::error!("Failed to save game {}: {}", room_id, e);
        } else {
            log::debug!("Saved game state for room {}", room_id);
        }
    }

    /// Load a game snapshot by room ID.
    pub async fn load_game(&self, room_id: &str) -> Option<GameSnapshot> {
        match self.collection.find_one(doc! { "room_id": room_id }).await {
            Ok(snapshot) => snapshot,
            Err(e) => {
                log::error!("Failed to load game {}: {}", room_id, e);
                None
            }
        }
    }

    /// Load all active (non-finished) game snapshots. Used on server startup.
    pub async fn load_all_active_games(&self) -> Vec<GameSnapshot> {
        use futures_util::StreamExt;
        let filter = doc! { "phase": { "$ne": "Finished" } };
        match self.collection.find(filter).await {
            Ok(mut cursor) => {
                let mut results = Vec::new();
                while let Some(Ok(snap)) = cursor.next().await {
                    results.push(snap);
                }
                results
            }
            Err(e) => {
                log::error!("Failed to load active games: {}", e);
                Vec::new()
            }
        }
    }

    /// Delete a game snapshot (call when game finishes).
    pub async fn delete_game(&self, room_id: &str) {
        if let Err(e) = self.collection.delete_one(doc! { "room_id": room_id }).await {
            log::error!("Failed to delete game {}: {}", room_id, e);
        }
    }
}

// ─── Conversion helpers ───────────────────────────────────────────────────────

pub fn game_to_snapshot(room_id: &str, game: &Game) -> GameSnapshot {
    let board = game
        .board
        .squares
        .iter()
        .map(|row| {
            row.iter()
                .map(|sq| SquareSnapshot {
                    row: sq.row,
                    col: sq.col,
                    premium: format!("{:?}", sq.premium),
                    tile: sq.tile.clone(),
                    owner: sq.owner,
                })
                .collect()
        })
        .collect();

    let players = game
        .players
        .iter()
        .map(|p| PlayerSnapshot {
            id: p.id.clone(),
            name: p.name.clone(),
            score: p.score,
            rack: p.rack.iter().map(|t| TileSnapshot { letter: t.letter.clone(), value: t.value }).collect(),
            ready: p.ready,
            is_bot: p.is_bot,
            bot_level: p.bot_level.clone(),
        })
        .collect();

    let tile_bag = game
        .tile_bag
        .iter()
        .map(|t| TileSnapshot { letter: t.letter.clone(), value: t.value })
        .collect();

    let phase = match game.phase {
        GamePhase::Lobby => "Lobby",
        GamePhase::Playing => "Playing",
        GamePhase::Finished => "Finished",
    }
    .to_string();

    let updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    GameSnapshot {
        room_id: room_id.to_string(),
        board,
        players,
        tile_bag,
        current_turn: game.current_turn,
        phase,
        consecutive_passes: game.consecutive_passes,
        winner: game.winner.clone(),
        updated_at,
    }
}

pub fn snapshot_to_game(snapshot: &GameSnapshot, dictionary: Dictionary) -> Game {
    let squares: Vec<Vec<Square>> = snapshot
        .board
        .iter()
        .map(|row| {
            row.iter()
                .map(|sq| Square {
                    row: sq.row,
                    col: sq.col,
                    premium: parse_premium(&sq.premium),
                    tile: sq.tile.clone(),
                    owner: sq.owner,
                })
                .collect()
        })
        .collect();

    let board = Board {
        size: squares.len(),
        squares,
    };

    let players: Vec<Player> = snapshot
        .players
        .iter()
        .map(|p| Player {
            id: p.id.clone(),
            name: p.name.clone(),
            score: p.score,
            rack: p.rack.iter().map(|t| Tile { letter: t.letter.clone(), value: t.value }).collect(),
            ready: p.ready,
            is_bot: p.is_bot,
            bot_level: p.bot_level.clone(),
        })
        .collect();

    let tile_bag: Vec<Tile> = snapshot
        .tile_bag
        .iter()
        .map(|t| Tile { letter: t.letter.clone(), value: t.value })
        .collect();

    let phase = match snapshot.phase.as_str() {
        "Playing" => GamePhase::Playing,
        "Finished" => GamePhase::Finished,
        _ => GamePhase::Lobby,
    };

    Game {
        board,
        players,
        tile_bag,
        current_turn: snapshot.current_turn,
        phase,
        consecutive_passes: snapshot.consecutive_passes,
        dictionary,
        winner: snapshot.winner.clone(),
        tiles_placed_this_turn: Vec::new(),
    }
}

fn parse_premium(s: &str) -> Premium {
    match s {
        "DL" => Premium::DL,
        "TL" => Premium::TL,
        "DW" => Premium::DW,
        "TW" => Premium::TW,
        _ => Premium::Normal,
    }
}
