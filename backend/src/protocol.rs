//! WebSocket protocol messages between client and server.

use serde::{Deserialize, Serialize};

/// Messages from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Sign up a new account
    SignUp {
        email: String,
        password: String,
        display_name: String,
    },
    /// Sign in to existing account
    SignIn {
        email: String,
        password: String,
    },
    /// Create a new game room
    CreateRoom {
        player_name: String,
    },
    /// Join an existing game room
    JoinRoom {
        room_id: String,
        player_name: String,
    },
    /// Ready to start
    Ready,
    /// Place tiles on the board
    PlaceTiles {
        placements: Vec<TilePlacement>,
    },
    /// Exchange tiles from rack
    ExchangeTiles {
        letters: Vec<String>,
    },
    /// Skip/pass turn
    PassTurn,
    /// Resign from game
    Resign,
    /// Chat message
    Chat {
        message: String,
    },
}

/// A single tile placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePlacement {
    pub row: usize,
    pub col: usize,
    pub letter: String,
}

/// Messages from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Room created successfully
    RoomCreated {
        room_id: String,
        player_id: String,
        players: Vec<PlayerInfo>,
    },
    /// Joined a room
    RoomJoined {
        room_id: String,
        player_id: String,
        players: Vec<PlayerInfo>,
    },
    /// Player joined the room
    PlayerJoined {
        player: PlayerInfo,
    },
    /// Game has started
    GameStarted {
        board: Vec<Vec<BoardSquare>>,
        players: Vec<PlayerInfo>,
        current_turn: u8,
    },
    /// Your tiles (sent to each player)
    YourTiles {
        tiles: Vec<String>,
    },
    /// A move was made
    MoveMade {
        player_id: String,
        placements: Vec<TilePlacement>,
        score: u32,
        words_formed: Vec<String>,
    },
    /// Board state update
    BoardUpdate {
        board: Vec<Vec<BoardSquare>>,
        scores: Vec<u32>,
        current_turn: u8,
        tiles_remaining: usize,
    },
    /// Turn notification
    YourTurn,
    /// Draw result for determining first player
    DrawResult {
        draws: Vec<DrawEntry>,
        first_player_id: String,
    },
    /// Player passed
    PlayerPassed {
        player_id: String,
    },
    /// Tiles exchanged
    TilesExchanged {
        player_id: String,
        count: usize,
    },
    /// Game over
    GameOver {
        winner: Option<String>,
        final_scores: Vec<u32>,
        reason: String,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Authentication success
    AuthSuccess {
        token: String,
        email: String,
        display_name: String,
    },
    /// Authentication error
    AuthError {
        message: String,
    },
    /// Chat message relayed
    Chat {
        player_id: String,
        player_name: String,
        message: String,
    },
    /// Room state for reconnection
    RoomState {
        room_id: String,
        players: Vec<PlayerInfo>,
        board: Vec<Vec<BoardSquare>>,
        scores: Vec<u32>,
        current_turn: u8,
        tiles_remaining: usize,
        game_started: bool,
        game_over: bool,
        winner: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub tile_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSquare {
    pub row: usize,
    pub col: usize,
    pub premium: String, // "DL", "TL", "DW", "TW", ""
    pub tile: Option<String>,
    pub owner: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawEntry {
    pub player_id: String,
    pub player_name: String,
    pub letter: String,
}
