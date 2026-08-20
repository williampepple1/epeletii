"use client";

import React, { useState, useEffect } from "react";
import { useGameStore } from "@/store/gameStore";

const BOT_DESCRIPTIONS = {
  "Curlew (Okolo)": "Simple, playful, and welcoming Curlew. Represents the natural origins of the land. Plays short 2-3 letter words, misses premium squares.",
  "Ottam tuwo": "Unpredictable popular masquerade. Central tile placements, complex board overlays, tries unexpected moves to raise the roof.",
  "King Jaja": "Competitive tactician, rack balancer. Plays high-value consonants on multipliers and blocks high-scoring lanes.",
  "Queen Kambasa": "Bold and decisive. Prioritizes Double Word and Triple Letter squares to dominate the board.",
  "Alagbarigha": "Strategic closed board defense. Prioritizes vowel management and blocking opponents' open lines.",
  "King Perekule I": "Economic tactician. Sophisticated spacing, hunts for 10-letter bingo bonuses (+50 points!).",
  "Ikuba": "Sacred guardian spirit of the realm. Unbeatable parallel placements, maximum tile efficiency, and track monitoring.",
};

export function Lobby() {
  const [nameInput, setNameInput] = useState("");
  const [joinId, setJoinId] = useState("");
  const [isReady, setIsReady] = useState(false);
  const [copied, setCopied] = useState(false);
  const [selectedBot, setSelectedBot] = useState("Curlew (Okolo)");

  const handleCopy = () => {
    if (roomId) {
      navigator.clipboard.writeText(roomId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const playerName = useGameStore((s) => s.playerName);
  const players = useGameStore((s) => s.players);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const error = useGameStore((s) => s.error);

  const setPlayerName = useGameStore((s) => s.setPlayerName);
  const createRoom = useGameStore((s) => s.createRoom);
  const joinRoom = useGameStore((s) => s.joinRoom);
  const spectateRoom = useGameStore((s) => s.spectateRoom);
  const ready = useGameStore((s) => s.ready);
  const roomId = useGameStore((s) => s.roomId);
  const activeRooms = useGameStore((s) => s.activeRooms || []);
  const getActiveRooms = useGameStore((s) => s.getActiveRooms);

  useEffect(() => {
    if (playerName && !roomId) {
      getActiveRooms();
      const interval = setInterval(getActiveRooms, 5000);
      return () => clearInterval(interval);
    }
  }, [playerName, roomId, getActiveRooms]);

  if (gameStarted) return null;

  return (
    <div className="max-w-md mx-auto bg-[var(--surface)] rounded-xl shadow-lg p-6 space-y-4">
      <div className="flex justify-center mb-2">
        <img
          src="/logo.jpg"
          alt="Epeletii Logo"
          className="w-24 h-24 rounded-2xl object-cover shadow-md border-2 border-amber-500/20"
        />
      </div>
      <h2 className="text-2xl font-bold text-center text-[var(--foreground)]">
        🦛 Epeletii
      </h2>
      <p className="text-center text-[var(--muted)] text-sm">
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
        <div className="space-y-4">
          <div className="space-y-3">
            <button
              onClick={() => createRoom()}
              className="w-full py-2.5 bg-amber-600 text-white rounded-lg font-semibold
                         hover:bg-amber-700 transition-colors text-base"
            >
              👥 Create Multiplayer Room
            </button>

            <div className="relative flex py-1 items-center">
              <div className="flex-grow border-t border-stone-200 dark:border-stone-800"></div>
              <span className="flex-shrink mx-3 text-[10px] text-stone-400 font-bold uppercase tracking-wider">or</span>
              <div className="flex-grow border-t border-stone-200 dark:border-stone-800"></div>
            </div>

            <div className="bg-stone-50 dark:bg-stone-850/30 p-4 rounded-xl border border-stone-200/60 dark:border-stone-800/80 space-y-3 text-left">
              <h3 className="text-xs font-black text-amber-600 dark:text-amber-500 uppercase tracking-wider flex items-center gap-1">
                🤖 Play with a Bot (Single Player)
              </h3>
              
              <div>
                <label className="block text-[9px] font-bold text-stone-400 dark:text-stone-500 uppercase tracking-wider mb-1">
                  Choose Opponent
                </label>
                <select
                  value={selectedBot}
                  onChange={(e) => setSelectedBot(e.target.value)}
                  className="w-full px-3 py-2 bg-white dark:bg-stone-900 border border-stone-300 dark:border-stone-750 rounded-lg text-sm text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-2 focus:ring-amber-500"
                >
                  <optgroup label="Novice / Beginner (Rating: 400 - 800)">
                    <option value="Curlew (Okolo)">Curlew (Okolo) - Novice</option>
                    <option value="Ottam tuwo">Ottam tuwo - Masquerade</option>
                  </optgroup>
                  <optgroup label="Intermediate (Rating: 900 - 1400)">
                    <option value="King Jaja">King Jaja - Tactical</option>
                    <option value="Queen Kambasa">Queen Kambasa - Aggressive</option>
                  </optgroup>
                  <optgroup label="Advanced / Expert (Rating: 1500 - 1900)">
                    <option value="Alagbarigha">Alagbarigha - Defensive</option>
                    <option value="King Perekule I">King Perekule I - Tactician</option>
                  </optgroup>
                  <optgroup label="Grandmaster (Rating: 2000+)">
                    <option value="Ikuba">Ikuba - Guardian Spirit</option>
                  </optgroup>
                </select>
              </div>

              {/* Bot Profile Description */}
              <div className="p-2.5 bg-white dark:bg-stone-900/50 rounded-lg border border-stone-200/40 dark:border-stone-800/40">
                <p className="text-[11px] text-stone-500 dark:text-stone-400 leading-relaxed italic">
                  {BOT_DESCRIPTIONS[selectedBot as keyof typeof BOT_DESCRIPTIONS]}
                </p>
              </div>

              <button
                onClick={() => createRoom(selectedBot)}
                className="w-full py-2 bg-stone-700 hover:bg-stone-800 dark:bg-stone-800 dark:hover:bg-stone-755 text-white font-bold rounded-lg text-sm transition-colors shadow-xs"
              >
                🎮 Start Single Player Game
              </button>
            </div>

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

          {/* Active Rooms Directory */}
          <div className="pt-4 border-t border-stone-200 dark:border-stone-750 space-y-3">
            <h3 className="text-sm font-bold text-stone-700 dark:text-stone-300 flex items-center gap-1.5">
              🌍 Active Public Rooms
            </h3>
            
            {activeRooms.length === 0 ? (
              <div className="text-center py-6 px-4 bg-stone-50 dark:bg-stone-850/20 rounded-xl border border-dashed border-stone-200 dark:border-stone-850 text-xs text-stone-400">
                No active games currently. Create one to get started!
              </div>
            ) : (
              <div className="space-y-2 max-h-60 overflow-y-auto pr-1">
                {activeRooms.map((room) => {
                  const isFull = room.player_count >= room.max_players;
                  const canJoin = !room.game_started && !isFull;
                  
                  return (
                    <div
                      key={room.id}
                      className="flex items-center justify-between p-3 bg-stone-50 dark:bg-stone-850/50 rounded-xl border border-stone-200/60 dark:border-stone-800/80 hover:shadow-xs transition-all"
                    >
                      <div className="space-y-0.5 text-left">
                        <p className="text-sm font-bold text-stone-800 dark:text-stone-250 truncate max-w-[180px]">
                          {room.name}
                        </p>
                        <div className="flex items-center gap-2 text-[10px] font-semibold text-stone-500">
                          <span className={`px-1.5 py-0.5 rounded-full ${room.game_started ? "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300" : "bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300"}`}>
                            {room.game_started ? "In Progress" : "Lobby"}
                          </span>
                          <span>•</span>
                          <span>{room.player_count}/{room.max_players} players</span>
                        </div>
                      </div>

                      <div className="flex gap-2">
                        {canJoin ? (
                          <button
                            onClick={() => joinRoom(room.id)}
                            className="px-3.5 py-1.5 text-xs bg-amber-600 hover:bg-amber-700 text-white font-bold rounded-lg transition-colors shadow-xs"
                          >
                            Join
                          </button>
                        ) : (
                          <button
                            onClick={() => spectateRoom(room.id)}
                            className="px-3.5 py-1.5 text-xs bg-stone-600 hover:bg-stone-700 dark:bg-stone-800 dark:hover:bg-stone-700 text-white dark:text-stone-300 font-bold rounded-lg transition-colors shadow-xs"
                          >
                            Spectate
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <div className="bg-amber-50 dark:bg-amber-955/20 border border-amber-200 dark:border-amber-900/40 rounded-xl p-3 flex items-center justify-between gap-3 text-left">
            <div>
              <p className="text-[10px] uppercase tracking-wider text-amber-800 dark:text-amber-400 font-bold">Room Code</p>
              <p className="text-lg font-mono font-black text-amber-900 dark:text-amber-200 select-all mt-0.5">
                {roomId}
              </p>
            </div>
            <button
              onClick={handleCopy}
              className={`px-3.5 py-1.5 rounded-lg text-xs font-bold transition-all shadow-xs shrink-0 ${
                copied
                  ? "bg-green-600 text-white"
                  : "bg-amber-600 hover:bg-amber-700 text-white"
              }`}
            >
              {copied ? "✓ Copied" : "📋 Copy Code"}
            </button>
          </div>

          <div>
            <p className="text-sm font-medium text-stone-600 mb-2">
              Players ({players.length}/4)
            </p>
            <div className="space-y-1">
              {players.map((p) => (
                <div
                  key={p.id}
                  className="flex items-center gap-2 px-3 py-2 bg-stone-50 dark:bg-stone-850/50 rounded-lg border border-stone-200/40 dark:border-stone-800/40"
                >
                  <div className="w-2 h-2 rounded-full bg-green-500" />
                  <span className="text-stone-800 dark:text-stone-200 flex items-center gap-1.5 font-semibold">
                    {p.name}
                    {p.is_bot && (
                      <span className="px-1 py-0.2 rounded-md bg-stone-200 dark:bg-stone-850 text-[9px] font-bold text-stone-600 dark:text-stone-400">BOT</span>
                    )}
                  </span>
                  {p.id === useGameStore.getState().playerId && (
                    <span className="text-xs text-stone-400">(you)</span>
                  )}
                </div>
              ))}
            </div>
          </div>

          <button
            onClick={() => { ready(); setIsReady(true); }}
            disabled={players.length < 2 || isReady}
            className={`w-full py-3 rounded-lg font-semibold transition-colors ${
              isReady
                ? "bg-green-100 text-green-700 ring-2 ring-green-400 cursor-default"
                : "bg-green-600 text-white hover:bg-green-700 disabled:opacity-40"
            }`}
          >
            {isReady ? "✓ Ready!" : "Ready"}
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
