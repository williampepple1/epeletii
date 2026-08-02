"use client";

import React, { useState } from "react";
import { useGameStore } from "@/store/gameStore";

export function DictionarySidebar() {
  const [search, setSearch] = useState("");
  const activeDefinitions = useGameStore((s) => s.activeDefinitions);
  const lastWords = useGameStore((s) => s.lastWords);
  const lookupWord = useGameStore((s) => s.lookupWord);
  const lastScore = useGameStore((s) => s.lastScore);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (!search.trim()) return;
    lookupWord(search.trim());
  };

  const cleanSearch = search.trim().toLowerCase();
  const searchResults = activeDefinitions[cleanSearch] || [];

  return (
    <div className="bg-[var(--surface)] border border-stone-200 dark:border-stone-700 rounded-xl shadow-lg p-4 space-y-5 w-full">
      <div className="border-b border-stone-100 dark:border-stone-700 pb-3">
        <h3 className="text-lg font-bold text-[var(--foreground)] flex items-center gap-1.5">
          📖 Ibani Dictionary
        </h3>
        <p className="text-xs text-stone-500">
          Learn meanings of Ibani words during gameplay
        </p>
      </div>

      {/* Dictionary Search Form */}
      <form onSubmit={handleSearch} className="space-y-2">
        <label className="text-xs font-semibold text-stone-600 dark:text-stone-400">
          Search word
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="e.g. abaji, ábá"
            className="flex-1 px-3 py-1.5 text-sm rounded-lg border border-stone-300 dark:border-stone-600
                       bg-white dark:bg-stone-700 text-stone-800 dark:text-stone-200
                       placeholder:text-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
          />
          <button
            type="submit"
            className="px-3 py-1.5 text-sm bg-amber-600 text-white rounded-lg font-medium hover:bg-amber-700 transition-colors"
          >
            Search
          </button>
        </div>
      </form>

      {/* Search results */}
      {search.trim() !== "" && (
        <div className="space-y-2">
          <p className="text-xs font-semibold text-stone-600 dark:text-stone-400">
            Search Results for "{search}"
          </p>
          {searchResults.length > 0 ? (
            <div className="space-y-2">
              {searchResults.map((def, idx) => (
                <div
                  key={idx}
                  className="bg-amber-50/50 dark:bg-stone-750 border border-amber-100 dark:border-stone-700 rounded-lg p-3 space-y-1"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-amber-900 dark:text-amber-400 text-sm">
                      {def.word}
                    </span>
                    {def.pos && (
                      <span className="text-[10px] uppercase tracking-wider bg-amber-100 dark:bg-amber-900/50 text-amber-800 dark:text-amber-300 px-1.5 py-0.5 rounded font-mono font-bold">
                        {def.pos}
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-stone-700 dark:text-stone-300">
                    {def.meaning}
                  </p>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-xs text-stone-400 italic">No definition found in database</p>
          )}
        </div>
      )}

      {/* Words Formed In Last Move */}
      {lastWords.length > 0 && (
        <div className="space-y-3 pt-2 border-t border-stone-100 dark:border-stone-700">
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold text-stone-600 dark:text-stone-400">
              Words Formed Last Move
            </span>
            <span className="text-xs font-bold text-green-600">
              +{lastScore} pts
            </span>
          </div>

          <div className="space-y-2">
            {lastWords.map((word, wIdx) => {
              const lower = word.toLowerCase();
              const defs = activeDefinitions[lower] || [];

              return (
                <div key={wIdx} className="bg-stone-50 dark:bg-stone-750/30 border border-stone-200 dark:border-stone-700/80 rounded-lg p-2.5 space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-stone-800 dark:text-stone-200 text-sm">
                      {word}
                    </span>
                    {defs.length === 0 && (
                      <span className="text-[9px] text-stone-400 italic">Loading meaning...</span>
                    )}
                  </div>

                  {defs.length > 0 ? (
                    <div className="space-y-1.5">
                      {defs.map((def, dIdx) => (
                        <div key={dIdx} className="text-xs border-t border-dashed border-stone-200 dark:border-stone-700 pt-1.5 first:border-0 first:pt-0">
                          <div className="flex items-center justify-between mb-0.5">
                            <span className="font-medium text-amber-800 dark:text-amber-400 text-xs">
                              {def.word}
                            </span>
                            {def.pos && (
                              <span className="text-[9px] font-bold text-stone-500 font-mono">
                                {def.pos}
                              </span>
                            )}
                          </div>
                          <p className="text-stone-600 dark:text-stone-300">
                            {def.meaning}
                          </p>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-[11px] text-stone-500 italic">
                      Definitions will appear when loaded
                    </p>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
