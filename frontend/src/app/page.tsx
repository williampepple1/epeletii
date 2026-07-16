"use client";

import React, { useEffect, useState } from "react";
import { useGameStore } from "@/store/gameStore";
import { GameBoard } from "@/components/GameBoard";
import { TileRack } from "@/components/TileRack";
import { Lobby } from "@/components/Lobby";
import { Scoreboard } from "@/components/Scoreboard";

export default function Home() {
  const connect = useGameStore((s) => s.connect);
  const connected = useGameStore((s) => s.connected);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const gameOver = useGameStore((s) => s.gameOver);
  const reset = useGameStore((s) => s.reset);
  const lastWords = useGameStore((s) => s.lastWords);
  const lastScore = useGameStore((s) => s.lastScore);
  const playerId = useGameStore((s) => s.playerId);
  const drawResult = useGameStore((s) => s.drawResult);
  const [showDraw, setShowDraw] = useState(false);

  useEffect(() => {
    connect();
  }, [connect]);

  // Show draw result overlay for 3 seconds
  useEffect(() => {
    if (drawResult && gameStarted) {
      setShowDraw(true);
      const timer = setTimeout(() => setShowDraw(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [drawResult, gameStarted]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-stone-100 to-amber-50 p-4">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-stone-800">⚕ Epeletii</h1>
            <p className="text-stone-500 text-sm">Ibani Scrabble</p>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5">
              <div
                className={`w-2.5 h-2.5 rounded-full ${
                  connected ? "bg-green-500" : "bg-red-500"
                }`}
              />
              <span className="text-xs text-stone-500">
                {connected ? "Connected" : "Disconnected"}
              </span>
            </div>
            {playerId && (
              <button
                onClick={reset}
                className="text-xs px-3 py-1.5 rounded-lg bg-stone-200 text-stone-600
                           hover:bg-stone-300 transition-colors"
              >
                Leave
              </button>
            )}
          </div>
        </div>

        {/* Draw result overlay */}
        {showDraw && drawResult && (
          <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
            <div className="bg-white rounded-2xl shadow-2xl p-8 max-w-sm w-full mx-4 text-center animate-bounce-in">
              <h3 className="text-xl font-bold text-stone-800 mb-4">Draw for First!</h3>
              <div className="space-y-3 mb-6">
                {drawResult.draws.map((d, i) => (
                  <div key={i} className="flex items-center justify-between px-4 py-2 bg-stone-50 rounded-lg">
                    <span className="font-medium text-stone-700">{d.player_name}</span>
                    <span className="text-2xl font-bold text-amber-700">
                      {d.letter === " " ? "☆" : d.letter}
                    </span>
                  </div>
                ))}
              </div>
              <div className="bg-amber-100 rounded-lg p-3">
                <p className="text-amber-800 font-bold">
                  🏆 {drawResult.draws.find((d) => d.player_id === drawResult.first_player_id)?.player_name} goes first!
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Main content */}
        {!gameStarted ? (
          <Lobby />
        ) : (
          <div className="flex gap-6 items-start">
            <div className="flex-1 flex flex-col items-center gap-4">
              <GameBoard />

              {/* Recent words notification */}
              {lastWords.length > 0 && (
                <div className="bg-green-100 border border-green-300 rounded-lg px-4 py-2 text-sm text-green-800 animate-fade-in">
                  {lastWords.join(", ")} — +{lastScore} pts
                </div>
              )}

              <TileRack />
            </div>

            <div className="w-64 shrink-0">
              <Scoreboard />

              {gameOver && (
                <button
                  onClick={reset}
                  className="mt-4 w-full py-3 bg-amber-600 text-white rounded-lg font-semibold
                             hover:bg-amber-700 transition-colors"
                >
                  Play Again
                </button>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
