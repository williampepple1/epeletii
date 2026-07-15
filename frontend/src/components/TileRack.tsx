"use client";

import React from "react";
import { useGameStore } from "@/store/gameStore";

const TILE_BG: Record<string, string> = {
  a: "bg-amber-100",
  i: "bg-amber-100",
  e: "bg-amber-100",
  o: "bg-amber-100",
  u: "bg-amber-100",
  ị: "bg-amber-200",
  ẹ: "bg-amber-200",
  ọ: "bg-amber-200",
  ụ: "bg-amber-200",
  ḅ: "bg-amber-200",
  " ": "bg-gray-200 border-dashed",
};

export function TileRack() {
  const myTiles = useGameStore((s) => s.myTiles);
  const selectedTile = useGameStore((s) => s.selectedTile);
  const selectTile = useGameStore((s) => s.selectTile);
  const placeTiles = useGameStore((s) => s.placeTiles);
  const clearPlacements = useGameStore((s) => s.clearPlacements);
  const passTurn = useGameStore((s) => s.passTurn);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const pendingPlacements = useGameStore((s) => s.pendingPlacements);
  const myPlayer = useGameStore((s) =>
    s.players.find((p) => p.id === s.playerId)
  );
  const tilesRemaining = useGameStore((s) => s.tilesRemaining);

  if (!gameStarted) return null;

  const canSubmit = pendingPlacements.length > 0;
  const canClear = pendingPlacements.length > 0;

  return (
    <div className="flex flex-col items-center gap-3">
      {/* Pending tile count */}
      {pendingPlacements.length > 0 && (
        <div className="text-sm text-amber-700 font-medium">
          {pendingPlacements.length} tile{pendingPlacements.length > 1 ? "s" : ""} placed — click Submit to play
        </div>
      )}

      {/* Tile rack */}
      <div className="flex gap-1.5 items-center">
        {myTiles.map((tile, i) => {
          const bg = TILE_BG[tile] || "bg-amber-100";
          const isSelected = selectedTile === i;
          const isBlank = tile === " ";

          return (
            <button
              key={i}
              onClick={() => selectTile(isSelected ? null : i)}
              className={`
                w-12 h-14 flex items-center justify-center text-xl font-bold rounded-md shadow-md
                ${bg}
                ${isSelected ? "ring-3 ring-amber-500 -translate-y-2" : ""}
                ${isBlank ? "text-gray-400 italic" : "text-stone-800"}
                transition-all duration-150 hover:-translate-y-1 cursor-pointer
              `}
            >
              {isBlank ? "?" : tile}
            </button>
          );
        })}
      </div>

      {/* Score & tiles left */}
      <div className="flex items-center gap-4 text-sm text-stone-600">
        <span>Tiles left: {tilesRemaining}</span>
        <span className="font-semibold">Score: {myPlayer?.score || 0}</span>
      </div>

      {/* Action buttons */}
      <div className="flex gap-3">
        <button
          onClick={clearPlacements}
          disabled={!canClear}
          className="px-4 py-2 rounded-lg bg-stone-200 text-stone-700 font-medium
                     hover:bg-stone-300 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Clear
        </button>
        <button
          onClick={placeTiles}
          disabled={!canSubmit}
          className="px-6 py-2 rounded-lg bg-amber-600 text-white font-semibold
                     hover:bg-amber-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Submit Move
        </button>
        <button
          onClick={passTurn}
          className="px-4 py-2 rounded-lg bg-stone-200 text-stone-700 font-medium
                     hover:bg-stone-300 transition-colors"
        >
          Pass
        </button>
      </div>
    </div>
  );
}
