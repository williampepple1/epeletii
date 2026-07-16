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

### Local development

```bash
# Terminal 1 — Backend
cd backend
cp .env.example .env   # edit MONGO_URI and JWT_SECRET if needed
cargo run

# Terminal 2 — Frontend
cd frontend
npm run dev
```

### Deploy to production

**Prerequisites:**
- Docker installed locally
- Terraform installed
- AWS CLI configured with the `personal` profile
- A MongoDB Atlas cluster (free tier works) or any reachable MongoDB

**Step 1 — Build and push the Docker image:**
```bash
# Build for x86 (t3a.nano)
docker build -t williampepple1/epeletii:latest .

# Push to Docker Hub (create account at hub.docker.com if needed)
docker push williampepple1/epeletii:latest
```

**Step 2 — Deploy infrastructure:**
```bash
cd infra
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your MongoDB URI and JWT secret

terraform init
terraform plan
terraform apply
```

**Step 3 — Configure frontend:**
```bash
# Get the server IP from Terraform output
terraform output ws_url
# → ws://<ip>:9001

# Set it in your frontend .env:
echo "NEXT_PUBLIC_WS_URL=ws://<ip>:9001" > frontend/.env.local
```

**Step 4 — Deploy frontend to Vercel:**
- Push the repo to GitHub
- Import the `frontend/` directory as a Vercel project
- Add `NEXT_PUBLIC_WS_URL` as an environment variable in Vercel dashboard

