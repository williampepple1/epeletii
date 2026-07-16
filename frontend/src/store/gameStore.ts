// Zustand game store for the Scrabble game

import { create } from "zustand";
import { BoardSquare, PlayerInfo, ServerMessage } from "@/types/game";
import { gameSocket } from "@/lib/websocket";

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
  placeOnBoard: (row: number, col: number) => void;
  dropOnBoard: (row: number, col: number, tileIndex: number) => void;
  clearPlacements: () => void;
  sendChat: (message: string) => void;
  connect: () => Promise<void>;
  reset: () => void;
}

const BOARD_SIZE = 15;

export const useGameStore = create<GameState>((set, get) => ({
  // Initial state
  connected: false,
  playerId: null,
  roomId: null,
  playerName: null,
  isLoggedIn: false,
  authToken: null,
  userEmail: null,
  userDisplayName: null,
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
    set({
      isLoggedIn: false,
      authToken: null,
      userEmail: null,
      userDisplayName: null,
      playerName: null,
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

  placeOnBoard: (row, col) => {
    const { selectedTile, myTiles, pendingPlacements, board } = get();
    if (selectedTile === null || selectedTile >= myTiles.length || !board) return;
    // Check square is empty on the real board and not already pending
    if (board[row][col].tile) return;
    if (pendingPlacements.some((p) => p.row === row && p.col === col)) return;
    const letter = myTiles[selectedTile];
    const newPending = [...pendingPlacements, { row, col, letter }];
    const newTiles = [...myTiles];
    newTiles.splice(selectedTile, 1);
    set({
      pendingPlacements: newPending,
      myTiles: newTiles,
      selectedTile: Math.min(selectedTile, newTiles.length - 1),
    });
  },

  clearPlacements: () => {
    const { pendingPlacements, myTiles } = get();
    const restored = [...myTiles, ...pendingPlacements.map((p) => p.letter)];
    set({ pendingPlacements: [], myTiles: restored, selectedTile: null });
  },

  dropOnBoard: (row, col, tileIndex) => {
    set({ selectedTile: tileIndex });
    get().placeOnBoard(row, col);
  },

  sendChat: (message) => {
    gameSocket.send({ type: "Chat", message });
  },

  connect: async () => {
    try {
      await gameSocket.connect();
      set({ connected: true, error: null });

      // Register handlers
      gameSocket.on("AuthSuccess", (msg) => {
        if (msg.type === "AuthSuccess") {
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
          set({ playerId: msg.player_id, roomId: msg.room_id, players: msg.players });
        }
      });

      gameSocket.on("RoomJoined", (msg) => {
        if (msg.type === "RoomJoined") {
          set({
            playerId: msg.player_id,
            roomId: msg.room_id,
            players: msg.players,
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
        }
      });

      gameSocket.on("YourTurn", () => {
        set({ yourTurn: true });
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
        }
      });

      gameSocket.on("Error", (msg) => {
        if (msg.type === "Error") {
          set({ error: msg.message });
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
    });
  },
}));
