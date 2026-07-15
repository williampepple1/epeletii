// Type definitions mirroring the Rust server protocol

export interface TilePlacement {
  row: number;
  col: number;
  letter: string;
}

export interface PlayerInfo {
  id: string;
  name: string;
  score: number;
  tile_count: number;
}

export interface BoardSquare {
  row: number;
  col: number;
  premium: string; // "DL", "TL", "DW", "TW", "Normal"
  tile: string | null;
  owner: number | null;
}

// Client -> Server messages
export type ClientMessage =
  | { type: "CreateRoom"; player_name: string }
  | { type: "JoinRoom"; room_id: string; player_name: string }
  | { type: "Ready" }
  | { type: "PlaceTiles"; placements: TilePlacement[] }
  | { type: "ExchangeTiles"; letters: string[] }
  | { type: "PassTurn" }
  | { type: "Resign" }
  | { type: "Chat"; message: string };

// Server -> Client messages
export type ServerMessage =
  | { type: "RoomCreated"; room_id: string; player_id: string; players: PlayerInfo[] }
  | { type: "RoomJoined"; room_id: string; player_id: string; players: PlayerInfo[] }
  | { type: "PlayerJoined"; player: PlayerInfo }
  | { type: "GameStarted"; board: BoardSquare[][]; players: PlayerInfo[]; current_turn: number }
  | { type: "YourTiles"; tiles: string[] }
  | { type: "MoveMade"; player_id: string; placements: TilePlacement[]; score: number; words_formed: string[] }
  | { type: "BoardUpdate"; board: BoardSquare[][]; scores: number[]; current_turn: number; tiles_remaining: number }
  | { type: "YourTurn" }
  | { type: "DrawResult"; draws: { player_id: string; player_name: string; letter: string }[]; first_player_id: string }
  | { type: "PlayerPassed"; player_id: string }
  | { type: "TilesExchanged"; player_id: string; count: number }
  | { type: "GameOver"; winner: string | null; final_scores: number[]; reason: string }
  | { type: "Error"; message: string }
  | { type: "Chat"; player_id: string; player_name: string; message: string }
  | { type: "RoomState"; room_id: string; players: PlayerInfo[]; board: BoardSquare[][]; scores: number[]; current_turn: number; tiles_remaining: number; game_started: boolean; game_over: boolean; winner: string | null };

// Premium square display mapping
export const PREMIUM_LABELS: Record<string, string> = {
  DL: "DL",
  TL: "TL",
  DW: "DW",
  TW: "TW",
  Normal: "",
};

export const PREMIUM_COLORS: Record<string, string> = {
  DL: "bg-blue-200 text-blue-800",
  TL: "bg-blue-500 text-white",
  DW: "bg-pink-200 text-pink-800",
  TW: "bg-red-500 text-white",
  Normal: "bg-amber-50",
};
