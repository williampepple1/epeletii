"use client";

import { useEffect, useState } from "react";
import { useGameStore } from "@/store/gameStore";

export function ScorePopup() {
  const lastWords = useGameStore((s) => s.lastWords);
  const lastScore = useGameStore((s) => s.lastScore);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (lastWords.length > 0) {
      setVisible(true);
      const t = setTimeout(() => setVisible(false), 2000);
      return () => clearTimeout(t);
    }
  }, [lastWords, lastScore]);

  if (!visible || lastWords.length === 0) return null;

  return (
    <div className="fixed top-1/3 left-1/2 -translate-x-1/2 z-30 pointer-events-none animate-float-up">
      <div className="bg-green-500 text-white px-6 py-3 rounded-2xl shadow-2xl text-center">
        <p className="text-lg font-bold">+{lastScore}</p>
        <p className="text-xs opacity-90">{lastWords.join(", ")}</p>
      </div>
    </div>
  );
}
