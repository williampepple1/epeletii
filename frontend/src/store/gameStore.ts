// Zustand game store for the Scrabble game

import { create } from "zustand";
import { BoardSquare, PlayerInfo, ServerMessage, WordDetail } from "@/types/game";
import { gameSocket } from "@/lib/websocket";
import { sounds } from "@/lib/sound";

interface GameState {
  // Connection
  connected: boolean;
  playerId: string | null;
  roomId: string | null;
  playerName: string | null;

  // Auth
  isLoggedIn: boolean;
  authToken: string | null;
  userEmail: string | null;
  userDisplayName: string | null;
  authLoading: boolean;
  authError: string | null;

  // Players
  players: PlayerInfo[];

  // Board
  board: BoardSquare[][] | null;
  currentTurn: number;
  tilesRemaining: number;

  // Player's tiles
  myTiles: string[];

  // Game phase
  gameStarted: boolean;
  gameOver: boolean;
  winner: string | null;
  gameOverReason: string;
  yourTurn: boolean; // set when server sends YourTurn, cleared on BoardUpdate

  // Selection (for placing tiles)
  selectedTile: number | null; // index in myTiles

  // Pending placements (tiles placed on board but not yet submitted)
  pendingPlacements: { row: number; col: number; letter: string }[];

  // Messages
  error: string | null;
  chatMessages: { playerId: string; playerName: string; message: string }[];
  lastWords: string[];
  lastScore: number;

  // Draw result
  drawResult: { draws: { player_id: string; player_name: string; letter: string }[]; first_player_id: string } | null;

  activeDefinitions: Record<string, WordDetail[]>;
  lookupHistory: string[];

  // Actions
  setPlayerName: (name: string) => void;
  signUp: (email: string, password: string, displayName: string) => void;
  signIn: (email: string, password: string) => void;
  logOut: () => void;
  createRoom: () => void;
  joinRoom: (roomId: string) => void;
  ready: () => void;
  placeTiles: () => void;
  passTurn: () => void;
  exchangeTiles: (letters: string[]) => void;
  selectTile: (index: number | null) => void;
  placeOnBoard: (row: number, col: number, tileIndex?: number) => void;
  dropOnBoard: (row: number, col: number, tileIndex: number) => void;
  /// Remove a single pending tile back to rack
  removePendingPlacement: (row: number, col: number) => void;
  clearPlacements: () => void;
  sendChat: (message: string) => void;
  lookupWord: (word: string) => void;
  shuffleRack: () => void;
  connect: () => Promise<void>;
  reset: () => void;
}

const isClient = typeof window !== "undefined";

const getLocalStorage = (key: string): string | null => {
  if (!isClient) return null;
  return localStorage.getItem(key);
};

export const useGameStore = create<GameState>((set, get) => ({
  // Initial state
  connected: false,
  playerId: getLocalStorage("playerId"),
  roomId: getLocalStorage("roomId"),
  playerName: getLocalStorage("userDisplayName"),
  isLoggedIn: !!getLocalStorage("authToken"),
  authToken: getLocalStorage("authToken"),
  userEmail: getLocalStorage("userEmail"),
  userDisplayName: getLocalStorage("userDisplayName"),
  authLoading: false,
  authError: null,
  players: [],
  board: null,
  currentTurn: 0,
  tilesRemaining: 170,
  myTiles: [],
  gameStarted: false,
  gameOver: false,
  winner: null,
  gameOverReason: "",
  yourTurn: false,
  selectedTile: null,
  pendingPlacements: [],
  error: null,
  chatMessages: [],
  lastWords: [],
  lastScore: 0,
  drawResult: null,
  activeDefinitions: {},
  lookupHistory: [],

  setPlayerName: (name) => set({ playerName: name }),

  signUp: (email, password, displayName) => {
    set({ authLoading: true, authError: null });
    gameSocket.send({ type: "SignUp", email, password, display_name: displayName });
  },

  signIn: (email, password) => {
    set({ authLoading: true, authError: null });
    gameSocket.send({ type: "SignIn", email, password });
  },

  logOut: () => {
    if (isClient) {
      localStorage.removeItem("authToken");
      localStorage.removeItem("userEmail");
      localStorage.removeItem("userDisplayName");
      localStorage.removeItem("roomId");
      localStorage.removeItem("playerId");
    }
    set({
      isLoggedIn: false,
      authToken: null,
      userEmail: null,
      userDisplayName: null,
      playerName: null,
      playerId: null,
      roomId: null,
      gameStarted: false,
      board: null,
      myTiles: [],
    });
  },

  createRoom: () => {
    const { playerName } = get();
    if (!playerName) return;
    gameSocket.send({ type: "CreateRoom", player_name: playerName });
  },

  joinRoom: (roomId) => {
    const { playerName } = get();
    if (!playerName) return;
    gameSocket.send({ type: "JoinRoom", room_id: roomId, player_name: playerName });
  },

  ready: () => {
    gameSocket.send({ type: "Ready" });
  },

  placeTiles: () => {
    const { pendingPlacements } = get();
    if (pendingPlacements.length === 0) return;
    gameSocket.send({
      type: "PlaceTiles",
      placements: pendingPlacements.map((p) => ({
        row: p.row,
        col: p.col,
        letter: p.letter,
      })),
    });
    set({ pendingPlacements: [], selectedTile: null });
  },

  passTurn: () => {
    gameSocket.send({ type: "PassTurn" });
  },

  exchangeTiles: (letters) => {
    gameSocket.send({ type: "ExchangeTiles", letters });
  },

  selectTile: (index) => set({ selectedTile: index }),

  placeOnBoard: (row, col, tileIndex) => {
    const { selectedTile: storeSelected, myTiles, pendingPlacements, board } = get();
    const index = tileIndex !== undefined ? tileIndex : storeSelected;
    if (index === null || index === undefined || index >= myTiles.length || !board) return;
    // Check square is empty on the real board and not already pending
    if (board[row][col].tile) return;
    if (pendingPlacements.some((p) => p.row === row && p.col === col)) return;
    const letter = myTiles[index];
    const newPending = [...pendingPlacements, { row, col, letter }];
    const newTiles = [...myTiles];
    newTiles.splice(index, 1);

    // Update selectedTile to a safe value
    let nextSelected = storeSelected;
    if (tileIndex === undefined) {
      nextSelected = Math.min(index, newTiles.length - 1);
    } else {
      if (storeSelected !== null) {
        if (index === storeSelected) {
          nextSelected = null;
        } else if (index < storeSelected) {
          nextSelected = storeSelected - 1;
        }
      }
    }

    set({
      pendingPlacements: newPending,
      myTiles: newTiles,
      selectedTile: nextSelected,
    });
  },

  clearPlacements: () => {
    const { pendingPlacements, myTiles } = get();
    const restored = [...myTiles, ...pendingPlacements.map((p) => p.letter)];
    set({ pendingPlacements: [], myTiles: restored, selectedTile: null });
  },

  removePendingPlacement: (row, col) => {
    const { pendingPlacements, myTiles } = get();
    const idx = pendingPlacements.findIndex((p) => p.row === row && p.col === col);
    if (idx === -1) return;
    const removed = pendingPlacements[idx];
    const newPending = [...pendingPlacements];
    newPending.splice(idx, 1);
    set({
      pendingPlacements: newPending,
      myTiles: [...myTiles, removed.letter],
    });
  },

  dropOnBoard: (row, col, tileIndex) => {
    get().placeOnBoard(row, col, tileIndex);
  },

  sendChat: (message) => {
    gameSocket.send({ type: "Chat", message });
  },

  lookupWord: (word) => {
    if (!word) return;
    const lower = word.trim().toLowerCase();
    if (get().activeDefinitions[lower]) return;
    gameSocket.send({ type: "LookupWord", word: lower });
  },

  shuffleRack: () => {
    const { myTiles } = get();
    if (myTiles.length <= 1) return;
    const shuffled = [...myTiles];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    set({ myTiles: shuffled, selectedTile: null });
  },

  connect: async () => {
    try {
      await gameSocket.connect();
      set({ connected: true, error: null });

      // Try to auto-rejoin if we have active roomId and playerId in localStorage
      const cachedRoomId = isClient ? localStorage.getItem("roomId") : null;
      const cachedPlayerId = isClient ? localStorage.getItem("playerId") : null;
      if (cachedRoomId && cachedPlayerId) {
        gameSocket.send({
          type: "RejoinRoom",
          room_id: cachedRoomId,
          player_id: cachedPlayerId,
        });
      }

      // Register handlers
      gameSocket.on("AuthSuccess", (msg) => {
        if (msg.type === "AuthSuccess") {
          if (isClient) {
            localStorage.setItem("authToken", msg.token);
            localStorage.setItem("userEmail", msg.email);
            localStorage.setItem("userDisplayName", msg.display_name);
          }
          set({
            isLoggedIn: true,
            authToken: msg.token,
            userEmail: msg.email,
            userDisplayName: msg.display_name,
            playerName: msg.display_name,
            authLoading: false,
            authError: null,
          });
        }
      });

      gameSocket.on("AuthError", (msg) => {
        if (msg.type === "AuthError") {
          set({ authLoading: false, authError: msg.message });
        }
      });

      gameSocket.on("RoomCreated", (msg) => {
        if (msg.type === "RoomCreated") {
          if (isClient) {
            localStorage.setItem("roomId", msg.room_id);
            localStorage.setItem("playerId", msg.player_id);
          }
          set({ playerId: msg.player_id, roomId: msg.room_id, players: msg.players });
        }
      });

      gameSocket.on("RoomJoined", (msg) => {
        if (msg.type === "RoomJoined") {
          if (isClient) {
            localStorage.setItem("roomId", msg.room_id);
            localStorage.setItem("playerId", msg.player_id);
          }
          set({
            playerId: msg.player_id,
            roomId: msg.room_id,
            players: msg.players,
          });
        }
      });

      gameSocket.on("RoomState", (msg) => {
        if (msg.type === "RoomState") {
          set({
            roomId: msg.room_id,
            players: msg.players,
            board: msg.board,
            currentTurn: msg.current_turn,
            tilesRemaining: msg.tiles_remaining,
            gameStarted: msg.game_started,
            gameOver: msg.game_over,
            winner: msg.winner,
            error: null,
            drawResult: null,
          });
        }
      });

      gameSocket.on("PlayerJoined", (msg) => {
        if (msg.type === "PlayerJoined") {
          set((state) => ({
            players: [...state.players, msg.player],
          }));
        }
      });

      gameSocket.on("GameStarted", (msg) => {
        if (msg.type === "GameStarted") {
          // Keep drawResult for the overlay — it's cleared after timeout
          set({
            gameStarted: true,
            board: msg.board,
            players: msg.players,
            currentTurn: msg.current_turn,
            error: null,
            yourTurn: false,
          });
        }
      });

      gameSocket.on("YourTiles", (msg) => {
        if (msg.type === "YourTiles") {
          set({ myTiles: msg.tiles });
        }
      });

      gameSocket.on("BoardUpdate", (msg) => {
        if (msg.type === "BoardUpdate") {
          set({
            board: msg.board,
            players: (get().players || []).map((p, i) => ({
              ...p,
              score: msg.scores[i] || p.score,
            })),
            currentTurn: msg.current_turn,
            tilesRemaining: msg.tiles_remaining,
            yourTurn: false, // not our turn anymore after update
            lastWords: [], // clear word notification on new state
            lastScore: 0,
            drawResult: null, // clear draw overlay
          });
        }
      });

      gameSocket.on("MoveMade", (msg) => {
        if (msg.type === "MoveMade") {
          set({
            lastWords: msg.words_formed,
            lastScore: msg.score,
          });
          sounds.moveSubmit();
          msg.words_formed.forEach((word) => {
            get().lookupWord(word);
          });
        }
      });

      gameSocket.on("WordDefinition", (msg) => {
        if (msg.type === "WordDefinition") {
          const lower = msg.word.trim().toLowerCase();
          const activeDefs = { ...get().activeDefinitions };
          activeDefs[lower] = msg.definitions;

          const history = [
            lower,
            ...get().lookupHistory.filter((w) => w !== lower)
          ].slice(0, 50);

          set({
            activeDefinitions: activeDefs,
            lookupHistory: history,
          });
        }
      });

      gameSocket.on("YourTurn", () => {
        set({ yourTurn: true });
        sounds.yourTurn();
      });

      gameSocket.on("DrawResult", (msg) => {
        if (msg.type === "DrawResult") {
          set({ drawResult: msg });
        }
      });

      gameSocket.on("PlayerPassed", () => {
        // Could show notification
      });

      gameSocket.on("GameOver", (msg) => {
        if (msg.type === "GameOver") {
          set({
            gameOver: true,
            winner: msg.winner,
            gameOverReason: msg.reason,
            players: (get().players || []).map((p, i) => ({
              ...p,
              score: msg.final_scores[i] || p.score,
            })),
          });
          sounds.gameOver();
        }
      });

      gameSocket.on("Error", (msg) => {
        if (msg.type === "Error") {
          set({ error: msg.message });
          sounds.invalidMove();
          setTimeout(() => set({ error: null }), 5000);
        }
      });

      gameSocket.on("Chat", (msg) => {
        if (msg.type === "Chat") {
          set((state) => ({
            chatMessages: [
              ...state.chatMessages,
              {
                playerId: msg.player_id,
                playerName: msg.player_name,
                message: msg.message,
              },
            ],
          }));
        }
      });
    } catch (e) {
      set({ error: "Failed to connect to game server" });
    }
  },

  reset: () => {
    if (isClient) {
      localStorage.removeItem("roomId");
      localStorage.removeItem("playerId");
    }
    gameSocket.disconnect();
    set({
      connected: false,
      playerId: null,
      roomId: null,
      players: [],
      board: null,
      currentTurn: 0,
      tilesRemaining: 170,
      myTiles: [],
      gameStarted: false,
      gameOver: false,
      winner: null,
      gameOverReason: "",
      pendingPlacements: [],
      selectedTile: null,
      error: null,
      chatMessages: [],
      lastWords: [],
      lastScore: 0,
      activeDefinitions: {},
      lookupHistory: [],
    });
  },
}));
