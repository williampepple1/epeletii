# ⚕ Epeletii — Multiplayer Ibani Scrabble

Ibani word for "scrabble" / "letter arrangement game".

A multiplayer online Scrabble game for the Ibani language (Bonny/Opobo, Rivers State, Nigeria).

## Architecture

```
epeletii/
├── backend/          # Rust WebSocket game server
│   ├── src/
│   │   ├── main.rs         # Server entrypoint, WebSocket handling
│   │   ├── board.rs        # 15×15 Scrabble board with premium squares
│   │   ├── dictionary.rs   # Word validation via Ibani-dictionary SQLite
│   │   ├── game.rs         # Game state machine, scoring, turn logic
│   │   ├── protocol.rs     # Client↔Server message types
│   │   ├── room.rs         # Game room management
│   │   └── tiles.rs        # Ibani letter tiles, distribution, points
│   └── Cargo.toml
├── frontend/         # Next.js + TypeScript + Zustand
│   └── src/
│       ├── app/            # Pages (main game page)
│       ├── components/     # GameBoard, TileRack, Lobby, Scoreboard
│       ├── lib/            # WebSocket connection manager
│       ├── store/          # Zustand game state store
│       └── types/          # Shared TypeScript types
└── README.md
```

## Stack

- **Backend**: Rust (tokio + tokio-tungstenite + rusqlite)
- **Frontend**: Next.js 16 + TypeScript + Tailwind v4 + Zustand
- **Protocol**: JSON over WebSocket
- **Dictionary**: Direct SQLite access to Ibani-dictionary.db

## Letter Distribution

170 tiles total across 28 letter types. Under-dotted vowels (ị, ẹ, ọ, ụ) and ḅ are distinct tiles. 2 blank tiles.

## Game Rules

- Standard 15×15 Scrabble board with premium squares
- 2-4 players per room
- First move must cover center (7,7)
- Words validated against the Ibani dictionary
- 50-point bonus for using all 7 tiles (bingo)
- Consecutive passes by all players ends the game

## Running

```bash
# Backend
cd backend
cargo run

# Frontend (separate terminal)
cd frontend
npm run dev
```
