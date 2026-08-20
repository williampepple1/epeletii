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
  const tilesRemaining = useGameStore((s) => s.tilesRemaining);
  const resign = useGameStore((s) => s.resign);
  const reset = useGameStore((s) => s.reset);

  const isSpectator = !players.some((p) => p.id === playerId);

  if (!gameStarted) return null;

  return (
    <div className="bg-[var(--surface)] rounded-xl shadow-lg p-4 space-y-3">
      <h3 className="text-lg font-bold text-[var(--foreground)]">Scoreboard</h3>

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
                <span className="text-stone-800 flex items-center gap-1.5">
                  {p.name}
                  {p.is_bot && (
                    <span className="px-1 py-0.2 rounded-md bg-stone-200 dark:bg-stone-800 text-[9px] font-bold text-stone-600 dark:text-stone-400">BOT</span>
                  )}
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

      <div className="pt-2 border-t border-stone-200 dark:border-stone-750 flex items-center justify-between text-xs text-stone-500 dark:text-stone-400">
        <span>Tiles in bag:</span>
        <span className="font-semibold px-2.5 py-0.5 rounded-full bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
          {tilesRemaining}
        </span>
      </div>

      {!gameOver && (
        isSpectator ? (
          <button
            onClick={reset}
            className="w-full py-1.5 text-xs border border-stone-250/20 dark:border-stone-800 text-stone-600 dark:text-stone-300
                       rounded-lg font-semibold hover:bg-stone-50 dark:hover:bg-stone-850/50 transition-colors"
          >
            🚪 Leave Room
          </button>
        ) : (
          <button
            onClick={() => {
              if (confirm("Are you sure you want to resign? Your score will be reset to 0 and all your points will be transferred to your opponent(s).")) {
                resign();
              }
            }}
            className="w-full py-1.5 text-xs border border-red-200 dark:border-red-900/60 text-red-600 dark:text-red-400
                       rounded-lg font-semibold hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors"
          >
            🏳️ Resign Game
          </button>
        )
      )}
    </div>
  );
}
