"use client";

import React from "react";
import { useGameStore } from "@/store/gameStore";
import { sounds } from "@/lib/sound";

const PREMIUM_BG: Record<string, string> = {
  TW: "bg-red-600 text-white",
  DW: "bg-pink-300 text-pink-900",
  TL: "bg-blue-500 text-white",
  DL: "bg-cyan-300 text-cyan-900",
  Normal: "bg-amber-50 text-amber-900",
};

const PREMIUM_LABEL: Record<string, string> = {
  TW: "TW",
  DW: "DW",
  TL: "TL",
  DL: "DL",
  Normal: "",
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

  if (!board || !gameStarted) {
    return (
      <div className="flex items-center justify-center h-96 bg-stone-100 rounded-lg">
        <p className="text-stone-500 text-lg">Waiting for game to start...</p>
      </div>
    );
  }

  const hasSelectedTile = selectedTile !== null && selectedTile < myTiles.length;
  const yourTurn = useGameStore((s) => s.yourTurn);
  const isPending = (r: number, c: number) =>
    pendingPlacements.some((p) => p.row === r && p.col === c);
  const pendingTile = (r: number, c: number) =>
    pendingPlacements.find((p) => p.row === r && p.col === c);

  return (
    <div className="inline-block bg-amber-800 p-2 rounded-xl shadow-2xl">
      <div
        className="grid gap-0.5"
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

            return (
              <div
                key={`${ri}-${ci}`}
                onClick={() => {
                  if (canUndo) { removePendingPlacement(ri, ci); sounds.tileReturn(); return; }
                  if (canPlace) { placeOnBoard(ri, ci); sounds.tilePlace(); }
                }}
                onDragOver={(e) => { if (canPlace) e.preventDefault(); }}
                onDrop={(e) => {
                  e.preventDefault();
                  const data = e.dataTransfer.getData("text/plain");
                  if (data && canPlace) {
                    const idx = parseInt(data, 10);
                    if (!isNaN(idx)) { dropOnBoard(ri, ci, idx); sounds.tilePlace(); }
                  }
                }}
                className={`
                  w-8 h-8 sm:w-10 sm:h-10 flex items-center justify-center text-sm font-bold rounded-sm select-none
                  ${pending ? "bg-amber-400 text-stone-900 shadow-lg ring-2 ring-amber-600" : bgClass}
                  ${occupied ? "text-stone-900 shadow-inner" : "text-[10px]"}
                  ${canPlace ? "hover:ring-2 hover:ring-amber-400 cursor-pointer" : ""}
                  ${canUndo ? "hover:ring-2 hover:ring-red-400 cursor-pointer" : ""}
                  ${ri === 7 && ci === 7 && !occupied && !pending ? "ring-2 ring-amber-400" : ""}
                  transition-all duration-100
                `}
                title={`${ri},${ci}${label ? ` (${label})` : ""}`}
              >
                {occupied ? (
                  <span className="text-lg font-semibold">{sq.tile}</span>
                ) : pending ? (
                  <span className="text-lg font-semibold">{pt?.letter}</span>
                ) : (
                  <span className="opacity-50">{label}</span>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
