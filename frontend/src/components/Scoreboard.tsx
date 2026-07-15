"use client";

import React from "react";
import { useGameStore } from "@/store/gameStore";

export function Scoreboard() {
  const players = useGameStore((s) => s.players);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const currentTurn = useGameStore((s) => s.currentTurn);
  const playerId = useGameStore((s) => s.playerId);
  const gameOver = useGameStore((s) => s.gameOver);
  const winner = useGameStore((s) => s.winner);
  const gameOverReason = useGameStore((s) => s.gameOverReason);

  if (!gameStarted) return null;

  return (
    <div className="bg-white rounded-xl shadow-lg p-4 space-y-3">
      <h3 className="text-lg font-bold text-stone-800">Scoreboard</h3>

      {gameOver && (
        <div className="bg-amber-100 border border-amber-300 rounded-lg p-3 text-center">
          <p className="text-amber-800 font-bold text-lg">
            {winner
              ? `🏆 ${players.find((p) => p.id === winner)?.name || "Someone"} wins!`
              : "Draw!"}
          </p>
          <p className="text-amber-600 text-sm">{gameOverReason}</p>
        </div>
      )}

      <div className="space-y-2">
        {players.map((p, i) => {
          const isCurrent = i === currentTurn && !gameOver;
          const isMe = p.id === playerId;

          return (
            <div
              key={p.id}
              className={`
                flex items-center justify-between px-3 py-2 rounded-lg
                ${isCurrent ? "bg-amber-100 ring-1 ring-amber-400" : "bg-stone-50"}
                ${isMe ? "font-semibold" : ""}
              `}
            >
              <div className="flex items-center gap-2">
                {isCurrent && (
                  <span className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
                )}
                <span className="text-stone-800">
                  {p.name}
                  {isMe && " (you)"}
                </span>
              </div>
              <span className="text-lg font-bold text-stone-900">
                {p.score}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
