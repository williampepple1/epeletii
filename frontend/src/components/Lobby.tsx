"use client";

import React, { useState } from "react";
import { useGameStore } from "@/store/gameStore";

export function Lobby() {
  const [nameInput, setNameInput] = useState("");
  const [joinId, setJoinId] = useState("");

  const playerName = useGameStore((s) => s.playerName);
  const players = useGameStore((s) => s.players);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const error = useGameStore((s) => s.error);

  const setPlayerName = useGameStore((s) => s.setPlayerName);
  const createRoom = useGameStore((s) => s.createRoom);
  const joinRoom = useGameStore((s) => s.joinRoom);
  const ready = useGameStore((s) => s.ready);
  const roomId = useGameStore((s) => s.roomId);

  if (gameStarted) return null;

  return (
    <div className="max-w-md mx-auto bg-white rounded-xl shadow-lg p-6 space-y-4">
      <h2 className="text-2xl font-bold text-center text-stone-800">
        ⚕ Epeletii
      </h2>
      <p className="text-center text-stone-500 text-sm">
        Multiplayer Ibani Scrabble
      </p>

      {error && (
        <div className="bg-red-100 border border-red-300 text-red-700 px-4 py-2 rounded-lg text-sm">
          {error}
        </div>
      )}

      {!playerName ? (
        <div className="space-y-3">
          <input
            type="text"
            placeholder="Your name"
            value={nameInput}
            onChange={(e) => setNameInput(e.target.value)}
            className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                       focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
            onKeyDown={(e) => {
              if (e.key === "Enter" && nameInput.trim()) {
                setPlayerName(nameInput.trim());
              }
            }}
          />
          <button
            onClick={() => setPlayerName(nameInput.trim())}
            disabled={!nameInput.trim()}
            className="w-full py-2 bg-amber-600 text-white rounded-lg font-semibold
                       hover:bg-amber-700 disabled:opacity-40 transition-colors"
          >
            Set Name
          </button>
        </div>
      ) : !roomId ? (
        <div className="space-y-3">
          <button
            onClick={createRoom}
            className="w-full py-3 bg-amber-600 text-white rounded-lg font-semibold
                       hover:bg-amber-700 transition-colors text-lg"
          >
            Create New Room
          </button>

          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Room ID to join"
              value={joinId}
              onChange={(e) => setJoinId(e.target.value)}
              className="flex-1 px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                         focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
            />
            <button
              onClick={() => joinRoom(joinId.trim())}
              disabled={!joinId.trim()}
              className="px-4 py-2 bg-stone-600 text-white rounded-lg font-medium
                         hover:bg-stone-700 disabled:opacity-40 transition-colors"
            >
              Join
            </button>
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <div className="bg-amber-50 border border-amber-200 rounded-lg p-3">
            <p className="text-sm text-amber-800 font-medium">Room Code</p>
            <p className="text-lg font-mono font-bold text-amber-900 select-all">
              {roomId}
            </p>
          </div>

          <div>
            <p className="text-sm font-medium text-stone-600 mb-2">
              Players ({players.length}/4)
            </p>
            <div className="space-y-1">
              {players.map((p) => (
                <div
                  key={p.id}
                  className="flex items-center gap-2 px-3 py-2 bg-stone-50 rounded-lg"
                >
                  <div className="w-2 h-2 rounded-full bg-green-500" />
                  <span className="text-stone-800">{p.name}</span>
                  {p.id === useGameStore.getState().playerId && (
                    <span className="text-xs text-stone-400">(you)</span>
                  )}
                </div>
              ))}
            </div>
          </div>

          <button
            onClick={ready}
            disabled={players.length < 2}
            className="w-full py-3 bg-green-600 text-white rounded-lg font-semibold
                       hover:bg-green-700 disabled:opacity-40 transition-colors"
          >
            Ready
          </button>

          {players.length < 2 && (
            <p className="text-xs text-stone-400 text-center">
              Need at least 2 players to start
            </p>
          )}
        </div>
      )}
    </div>
  );
}
