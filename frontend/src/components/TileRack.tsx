"use client";

import React, { useState } from "react";
import { useGameStore } from "@/store/gameStore";
import { sounds } from "@/lib/sound";

const TILE_VALUE: Record<string, number> = {
  a: 1, i: 1, e: 1, o: 1, u: 1,
  ị: 2, ẹ: 2, ọ: 2,
  n: 1, m: 1, r: 1,
  g: 2, s: 2, p: 2, b: 2, h: 2, k: 2,
  ụ: 3, d: 3, t: 3, w: 3, y: 3, ḅ: 3,
  l: 4, f: 4, j: 6, z: 6, v: 8, " ": 0,
};

export function TileRack() {
  const myTiles = useGameStore((s) => s.myTiles);
  const selectedTile = useGameStore((s) => s.selectedTile);
  const selectTile = useGameStore((s) => s.selectTile);
  const placeTiles = useGameStore((s) => s.placeTiles);
  const clearPlacements = useGameStore((s) => s.clearPlacements);
  const exchangeTiles = useGameStore((s) => s.exchangeTiles);
  const passTurn = useGameStore((s) => s.passTurn);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const pendingPlacements = useGameStore((s) => s.pendingPlacements);
  const yourTurn = useGameStore((s) => s.yourTurn);
  const myPlayer = useGameStore((s) => s.players.find((p) => p.id === s.playerId));
  const tilesRemaining = useGameStore((s) => s.tilesRemaining);
  const shuffleRack = useGameStore((s) => s.shuffleRack);

  const [exchangeMode, setExchangeMode] = useState(false);
  const [exchangeSet, setExchangeSet] = useState<Set<number>>(new Set());

  if (!gameStarted) return null;

  const canSubmit = pendingPlacements.length > 0 && yourTurn && !exchangeMode;
  const canClear = pendingPlacements.length > 0 && yourTurn && !exchangeMode;

  const toggleExchange = (i: number) => {
    const next = new Set(exchangeSet);
    if (next.has(i)) next.delete(i); else next.add(i);
    setExchangeSet(next);
  };

  const confirmExchange = () => {
    if (exchangeSet.size === 0) return;
    const letters = [...exchangeSet].map((i) => myTiles[i]);
    exchangeTiles(letters);
    setExchangeMode(false);
    setExchangeSet(new Set());
  };

  const cancelExchange = () => {
    setExchangeMode(false);
    setExchangeSet(new Set());
  };

  return (
    <div className="flex flex-col items-center gap-3.5 w-full max-w-lg mx-auto bg-stone-150/40 dark:bg-stone-900/30 p-4 rounded-xl border border-stone-250/20 dark:border-stone-850/50 backdrop-blur-xs">
      {/* Your Turn banner */}
      {yourTurn && !exchangeMode && (
        <div className="bg-linear-to-r from-amber-500 to-orange-500 text-white px-6 py-1.5 rounded-full text-sm font-black animate-pulse shadow-lg tracking-wide">
          ✦ Your Turn ✦
        </div>
      )}

      {/* Exchange mode header */}
      {exchangeMode && (
        <div className="bg-blue-50 dark:bg-blue-950/40 border border-blue-200 dark:border-blue-800 text-blue-800 dark:text-blue-300 px-4 py-2 rounded-lg text-sm font-semibold w-full text-center">
          Select tiles to exchange ({exchangeSet.size} selected)
        </div>
      )}

      {/* Pending tile count */}
      {pendingPlacements.length > 0 && yourTurn && !exchangeMode && (
        <div className="text-sm text-amber-700 dark:text-amber-400 font-semibold flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-amber-600 animate-ping" />
          {pendingPlacements.length} tile{pendingPlacements.length > 1 ? "s" : ""} placed — click Submit to play
        </div>
      )}

      {/* Tile rack */}
      <div className="flex gap-2 items-center flex-wrap justify-center p-3.5 bg-[#d2b48c] dark:bg-[#5c4033] rounded-xl border-b-8 border-[#8b5a2b] dark:border-[#3d251c] shadow-2xl relative">
        {/* Subtle wood sheen highlight */}
        <div className="absolute inset-0 bg-gradient-to-t from-transparent via-white/5 to-white/10 pointer-events-none rounded-xl" />
        
        {myTiles.map((tile, i) => {
          const isSelected = selectedTile === i && !exchangeMode;
          const isMarked = exchangeSet.has(i);
          const isBlank = tile === " ";
          const isUnderdotted = ["ị", "ẹ", "ọ", "ụ", "ḅ"].includes(tile);

          // Build premium tile class
          let tileClass = "w-12 h-14 relative flex flex-col items-center justify-center rounded-lg shadow-md border font-sans select-none transition-all duration-150 active:scale-95";
          
          if (isSelected) {
            tileClass += " ring-3 ring-amber-500 dark:ring-amber-400 shadow-xl -translate-y-2 scale-105";
          } else if (yourTurn) {
            tileClass += " hover:-translate-y-1 hover:shadow-lg cursor-pointer";
          } else {
            tileClass += " opacity-60 cursor-default";
          }

          if (isMarked) {
            tileClass += " bg-blue-100 dark:bg-blue-900 border-blue-400 dark:border-blue-700 ring-2 ring-blue-500";
          } else if (isBlank) {
            tileClass += " bg-gradient-to-b from-stone-50 to-stone-250 dark:from-stone-800 dark:to-stone-900 border-stone-300 dark:border-stone-700 border-dashed";
          } else if (isUnderdotted) {
            // Warm rich wood tone
            tileClass += " bg-gradient-to-b from-[#f5e4c3] to-[#e1c587] dark:from-[#584531] dark:to-[#422f1c] border-[#cbb27a] dark:border-[#523d29]";
          } else {
            // Standard letter wood tone
            tileClass += " bg-gradient-to-b from-[#fcf7e8] to-[#ebdfbf] dark:from-[#4d3d2e] dark:to-[#382a1e] border-[#d2c09d] dark:border-[#463526]";
          }

          return (
            <button
              key={i}
              draggable={yourTurn && !exchangeMode}
              onDragStart={(e) => {
                if (!yourTurn || exchangeMode) { e.preventDefault(); return; }
                e.dataTransfer.setData("text/plain", String(i));
                e.dataTransfer.effectAllowed = "move";
                sounds.tilePickup();
              }}
              onClick={() => {
                if (!yourTurn) return;
                if (exchangeMode) { toggleExchange(i); return; }
                selectTile(isSelected ? null : i);
                if (!isSelected) sounds.tilePickup();
              }}
              className={tileClass}
            >
              <span className={`text-2xl font-black leading-none ${isBlank ? "text-stone-400/80 dark:text-stone-600 italic" : "text-stone-850 dark:text-orange-50 drop-shadow-xs"}`}>
                {isBlank ? "?" : tile}
              </span>
              <span className="absolute bottom-1 right-1.5 text-[10px] font-bold text-stone-500 dark:text-stone-400 leading-none">
                {TILE_VALUE[tile] !== undefined && TILE_VALUE[tile] > 0 ? TILE_VALUE[tile] : ""}
              </span>
              {isMarked && (
                <span className="absolute -top-1 -right-1 w-4 h-4 bg-blue-600 text-white text-[10px] rounded-full flex items-center justify-center font-bold">
                  ✓
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Score & tiles left */}
      <div className="flex items-center justify-between w-full px-1 text-sm font-semibold text-stone-600 dark:text-stone-400">
        <span>Tiles left in bag: <strong className="text-stone-800 dark:text-stone-200">{tilesRemaining}</strong></span>
        <span>Your Score: <strong className="text-stone-850 dark:text-orange-200 text-base">{myPlayer?.score || 0}</strong></span>
      </div>

      {/* Action buttons */}
      <div className="flex gap-2 flex-wrap justify-center w-full">
        {exchangeMode ? (
          <>
            <button onClick={cancelExchange} className="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 text-stone-700 dark:text-stone-300 font-medium hover:bg-stone-300 dark:hover:bg-stone-700 transition-colors">
              Cancel
            </button>
            <button onClick={confirmExchange} disabled={exchangeSet.size === 0}
              className="px-6 py-2 rounded-lg bg-blue-600 text-white font-semibold hover:bg-blue-700 disabled:opacity-40 transition-colors">
              Exchange {exchangeSet.size > 0 ? `(${exchangeSet.size})` : ""}
            </button>
          </>
        ) : (
          <>
            <button onClick={clearPlacements} disabled={!canClear}
              className="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 text-stone-700 dark:text-stone-300 font-semibold hover:bg-stone-300 dark:hover:bg-stone-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
              Clear
            </button>
            <button onClick={shuffleRack} disabled={myTiles.length <= 1}
              className="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 text-stone-700 dark:text-stone-300 font-semibold hover:bg-stone-300 dark:hover:bg-stone-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
              title="Shuffle your tiles"
            >
              🔄 Shuffle
            </button>
            <button onClick={placeTiles} disabled={!canSubmit}
              className="px-6 py-2 rounded-lg bg-amber-600 hover:bg-amber-700 dark:bg-amber-700 dark:hover:bg-amber-600 text-white font-bold disabled:opacity-40 disabled:cursor-not-allowed shadow-md hover:shadow-lg transition-all">
              Submit Move
            </button>
            <button onClick={() => { setExchangeMode(true); setExchangeSet(new Set()); }}
              disabled={!yourTurn || pendingPlacements.length > 0}
              className="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 text-stone-700 dark:text-stone-300 font-semibold hover:bg-stone-300 dark:hover:bg-stone-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
              Exchange
            </button>
            <button onClick={passTurn} disabled={!yourTurn}
              className="px-4 py-2 rounded-lg bg-stone-200 dark:bg-stone-800 text-stone-700 dark:text-stone-300 font-semibold hover:bg-stone-300 dark:hover:bg-stone-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
              Pass
            </button>
          </>
        )}
      </div>
    </div>
  );
}
