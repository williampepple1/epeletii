"use client";

import React, { useState, useEffect } from "react";
import { useGameStore } from "@/store/gameStore";
import { sounds } from "@/lib/sound";

const PREMIUM_BG: Record<string, string> = {
  TW: "bg-gradient-to-br from-red-500 to-red-700 dark:from-red-600 dark:to-red-900 border-red-400 dark:border-red-800 text-white font-bold",
  DW: "bg-gradient-to-br from-pink-300 to-pink-400 dark:from-pink-850 dark:to-pink-950 border-pink-200 dark:border-pink-900 text-pink-900 dark:text-pink-100 font-bold",
  TL: "bg-gradient-to-br from-blue-500 to-blue-600 dark:from-blue-600 dark:to-blue-900 border-blue-400 dark:border-blue-800 text-white font-bold",
  DL: "bg-gradient-to-br from-cyan-300 to-cyan-400 dark:from-cyan-850 dark:to-cyan-950 border-cyan-200 dark:border-cyan-800 text-cyan-900 dark:text-cyan-100 font-bold",
  Normal: "bg-stone-100 dark:bg-stone-800/80 border-stone-200 dark:border-stone-700/60 text-stone-400 dark:text-stone-600",
};

const PREMIUM_LABEL: Record<string, string> = {
  TW: "TW",
  DW: "DW",
  TL: "TL",
  DL: "DL",
  Normal: "",
};

const TILE_VALUE: Record<string, number> = {
  a: 1, i: 1, e: 1, o: 1, u: 1,
  ị: 2, ẹ: 2, ọ: 2,
  n: 1, m: 1, r: 1,
  g: 2, s: 2, p: 2, b: 2, h: 2, k: 2,
  ụ: 3, d: 3, t: 3, w: 3, y: 3, ḅ: 3,
  l: 4, f: 4, j: 6, z: 6, v: 8, " ": 0,
};

export function GameBoard() {
  const board = useGameStore((s) => s.board);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const selectedTile = useGameStore((s) => s.selectedTile);
  const myTiles = useGameStore((s) => s.myTiles);
  const pendingPlacements = useGameStore((s) => s.pendingPlacements);
  const placeOnBoard = useGameStore((s) => s.placeOnBoard);
  const dropOnBoard = useGameStore((s) => s.dropOnBoard);
  const removePendingPlacement = useGameStore((s) => s.removePendingPlacement);
  const clearPlacements = useGameStore((s) => s.clearPlacements);
  const placeTiles = useGameStore((s) => s.placeTiles);
  const yourTurn = useGameStore((s) => s.yourTurn);

  const [cursor, setCursor] = useState<{ row: number; col: number } | null>(null);

  const hasSelectedTile = selectedTile !== null && selectedTile < myTiles.length;
  
  const isPending = (r: number, c: number) =>
    pendingPlacements.some((p) => p.row === r && p.col === c);
  const pendingTile = (r: number, c: number) =>
    pendingPlacements.find((p) => p.row === r && p.col === c);

  // Keyboard navigation & typing controls
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (document.activeElement?.tagName === "INPUT" || document.activeElement?.tagName === "TEXTAREA") {
        return;
      }
      
      if (!gameStarted || !yourTurn || !board) return;

      if (!cursor) {
        if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "w", "a", "s", "d", "W", "A", "S", "D"].includes(e.key)) {
          setCursor({ row: 7, col: 7 });
          e.preventDefault();
        }
        return;
      }

      const { row, col } = cursor;

      switch (e.key) {
        case "ArrowUp":
        case "w":
        case "W":
          setCursor({ row: Math.max(0, row - 1), col });
          e.preventDefault();
          break;
        case "ArrowDown":
        case "s":
        case "S":
          setCursor({ row: Math.min(14, row + 1), col });
          e.preventDefault();
          break;
        case "ArrowLeft":
        case "a":
        case "A":
          setCursor({ row, col: Math.max(0, col - 1) });
          e.preventDefault();
          break;
        case "ArrowRight":
        case "d":
        case "D":
          setCursor({ row, col: Math.min(14, col + 1) });
          e.preventDefault();
          break;
        case "Escape":
          clearPlacements();
          sounds.tileReturn();
          setCursor(null);
          e.preventDefault();
          break;
        case "Backspace":
          if (isPending(row, col)) {
            removePendingPlacement(row, col);
            sounds.tileReturn();
          }
          e.preventDefault();
          break;
        case "Enter":
          if (pendingPlacements.length > 0) {
            placeTiles();
          }
          e.preventDefault();
          break;
        case " ":
          // Match blank tile
          const blankIdx = myTiles.findIndex((t) => t === " ");
          if (blankIdx !== -1) {
            if (!board[row][col].tile && !isPending(row, col)) {
              dropOnBoard(row, col, blankIdx);
              sounds.tilePlace();
              setCursor({ row, col: Math.min(14, col + 1) });
            }
          }
          e.preventDefault();
          break;
        default:
          // Check if it's a valid Ibani letter in our rack
          const letter = e.key.toLowerCase();
          const rackIndex = myTiles.findIndex((t) => t.toLowerCase() === letter);
          if (rackIndex !== -1) {
            if (!board[row][col].tile && !isPending(row, col)) {
              dropOnBoard(row, col, rackIndex);
              sounds.tilePlace();
              
              // Automatically advance cursor
              let nextCol = Math.min(14, col + 1);
              let nextRow = row;
              if (pendingPlacements.length > 0) {
                const rows = pendingPlacements.map((p) => p.row);
                const isDown = rows[0] !== row;
                if (isDown) {
                  nextRow = Math.min(14, row + 1);
                  nextCol = col;
                }
              }
              setCursor({ row: nextRow, col: nextCol });
            }
          }
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [cursor, gameStarted, yourTurn, myTiles, pendingPlacements, board, clearPlacements, removePendingPlacement, dropOnBoard, placeTiles]);

  if (!board || !gameStarted) {
    return (
      <div className="flex items-center justify-center h-[420px] w-full max-w-[420px] sm:max-w-[500px] bg-stone-100 dark:bg-stone-850 rounded-xl border border-stone-250 dark:border-stone-750">
        <p className="text-stone-500 dark:text-stone-400 text-lg font-semibold animate-pulse">Waiting for game to start...</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-2 w-full">
      {/* Keyboard Controls Tip */}
      {yourTurn && (
        <p className="text-[11px] text-stone-500 dark:text-stone-400 italic">
          💡 Keyboard controls: Select square and type letters, backspace to undo, arrow keys to move.
        </p>
      )}
      
      <div className="inline-block bg-[#8b5a2b] dark:bg-[#3d251c] p-2 sm:p-3 rounded-2xl shadow-2xl border-4 border-[#5c3e21] dark:border-[#2b1912]">
        <div
          className="grid gap-0.5 sm:gap-1 bg-[#5c3e21] dark:bg-[#2b1912]"
          style={{ gridTemplateColumns: `repeat(${board.length}, minmax(0, 1fr))` }}
        >
          {board.map((row, ri) =>
            row.map((sq, ci) => {
              const bgClass = PREMIUM_BG[sq.premium] || PREMIUM_BG.Normal;
              const label = PREMIUM_LABEL[sq.premium];
              const occupied = sq.tile !== null;
              const pending = isPending(ri, ci);
              const pt = pendingTile(ri, ci);
              const canPlace = !occupied && !pending && hasSelectedTile && yourTurn;
              const canUndo = pending && yourTurn;
              const isCursor = cursor?.row === ri && cursor?.col === ci;

              // Compute square classes
              let sqClass = "w-[22px] h-[22px] xs:w-7 xs:h-7 sm:w-10 sm:h-10 flex flex-col items-center justify-center relative select-none rounded-xs sm:rounded-sm transition-all duration-100 border text-center font-sans";
              
              if (occupied) {
                sqClass += " bg-gradient-to-b from-[#fbf8f0] to-[#e5d9ba] dark:from-[#493a2a] dark:to-[#33261a] border-[#c0ab7b] dark:border-[#453424] text-stone-850 dark:text-orange-50 font-black shadow-md cursor-default";
              } else if (pending) {
                sqClass += " bg-gradient-to-b from-amber-400 to-amber-500 dark:from-amber-600 dark:to-amber-700 border-2 border-amber-600 dark:border-amber-500 text-stone-900 dark:text-orange-50 font-black shadow-md cursor-pointer animate-fade-in";
              } else {
                sqClass += ` ${bgClass}`;
                if (canPlace) {
                  sqClass += " hover:ring-2 hover:ring-amber-400/80 hover:scale-95 cursor-pointer";
                }
                if (canUndo) {
                  sqClass += " hover:ring-2 hover:ring-red-400 cursor-pointer";
                }
              }

              if (isCursor) {
                sqClass += " ring-3 ring-amber-500 dark:ring-amber-400 ring-offset-1 dark:ring-offset-stone-900 scale-95 z-20 shadow-lg";
              }

              return (
                <div
                  key={`${ri}-${ci}`}
                  onClick={() => {
                    setCursor({ row: ri, col: ci });
                    if (canUndo) { removePendingPlacement(ri, ci); sounds.tileReturn(); return; }
                    if (canPlace) { placeOnBoard(ri, ci); sounds.tilePlace(); }
                  }}
                  onDragOver={(e) => { if (canPlace) e.preventDefault(); }}
                  onDrop={(e) => {
                    e.preventDefault();
                    const data = e.dataTransfer.getData("text/plain");
                    if (data && canPlace) {
                      const idx = parseInt(data, 10);
                      if (!isNaN(idx)) {
                        dropOnBoard(ri, ci, idx);
                        sounds.tilePlace();
                        setCursor({ row: ri, col: ci });
                      }
                    }
                  }}
                  className={sqClass}
                  title={`${ri},${ci}${label ? ` (${label})` : ""}`}
                >
                  {occupied ? (
                    <>
                      <span className="text-xs xs:text-sm sm:text-base font-black leading-none drop-shadow-xs">{sq.tile}</span>
                      <span className="absolute bottom-[1px] right-0.5 sm:right-1 text-[7px] sm:text-[9px] font-bold text-stone-500 dark:text-stone-400 leading-none">
                        {TILE_VALUE[sq.tile?.toLowerCase() || ""] !== undefined ? TILE_VALUE[sq.tile?.toLowerCase() || ""] : ""}
                      </span>
                    </>
                  ) : pending ? (
                    <>
                      <span className="text-xs xs:text-sm sm:text-base font-black leading-none drop-shadow-xs">{pt?.letter}</span>
                      <span className="absolute bottom-[1px] right-0.5 sm:right-1 text-[7px] sm:text-[9px] font-bold text-stone-600 dark:text-stone-300 leading-none">
                        {TILE_VALUE[pt?.letter?.toLowerCase() || ""] !== undefined ? TILE_VALUE[pt?.letter?.toLowerCase() || ""] : ""}
                      </span>
                    </>
                  ) : (
                    <>
                      <span className="font-extrabold uppercase text-[7px] sm:text-[9px] tracking-tight">{label}</span>
                      {ri === 7 && ci === 7 && (
                        <span className="text-[10px] sm:text-xs leading-none mt-0.5 animate-pulse text-amber-500">★</span>
                      )}
                    </>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
