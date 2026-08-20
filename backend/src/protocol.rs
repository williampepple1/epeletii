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
        token: Option<String>,
        #[serde(default)]
        bot_level: Option<String>,
    },
    /// Join an existing game room
    JoinRoom {
        room_id: String,
        player_name: String,
        token: Option<String>,
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
    /// Lookup definitions for a word
    LookupWord {
        word: String,
    },
    /// Rejoin an existing room (reconnection)
    RejoinRoom {
        room_id: String,
        player_id: String,
    },
    /// Fetch the leaderboard
    GetLeaderboard,
    /// Fetch list of active/public game rooms
    GetActiveRooms,
    /// Spectate an existing game room
    SpectateRoom {
        room_id: String,
    },
    /// Request a password reset email
    ForgotPassword {
        email: String,
    },
    /// Reset password using a received token
    ResetPassword {
        email: String,
        token: String,
        new_password: String,
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
    /// Forgot password email sent notification
    ForgotPasswordSent,
    /// Reset password success notification
    ResetPasswordSuccess,
    /// Chat message relayed
    Chat {
        player_id: String,
        player_name: String,
        message: String,
    },
    /// Word definitions returned
    WordDefinition {
        word: String,
        definitions: Vec<WordDetail>,
    },
    /// Room state for reconnection
    RoomState {
        room_id: String,
        player_id: Option<String>,
        players: Vec<PlayerInfo>,
        board: Vec<Vec<BoardSquare>>,
        scores: Vec<u32>,
        current_turn: u8,
        tiles_remaining: usize,
        game_started: bool,
        game_over: bool,
        winner: Option<String>,
    },
    /// Leaderboard data
    Leaderboard {
        entries: Vec<LeaderboardEntry>,
    },
    /// Active/public game rooms data
    ActiveRooms {
        rooms: Vec<ActiveRoomInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRoomInfo {
    pub id: String,
    pub name: String,
    pub player_count: usize,
    pub max_players: usize,
    pub game_started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub display_name: String,
    pub games_played: u32,
    pub games_won: u32,
    pub total_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub tile_count: usize,
    #[serde(default)]
    pub is_bot: bool,
    #[serde(default)]
    pub bot_level: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDetail {
    pub word: String,
    pub pos: String,
    pub meaning: String,
}
