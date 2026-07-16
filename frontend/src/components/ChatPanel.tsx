"use client";

import React, { useState, useRef, useEffect } from "react";
import { useGameStore } from "@/store/gameStore";
import { sounds } from "@/lib/sound";

export function ChatPanel() {
  const [msg, setMsg] = useState("");
  const chatMessages = useGameStore((s) => s.chatMessages);
  const sendChat = useGameStore((s) => s.sendChat);
  const gameStarted = useGameStore((s) => s.gameStarted);
  const [open, setOpen] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (listRef.current) listRef.current.scrollTop = listRef.current.scrollHeight;
  }, [chatMessages]);

  if (!gameStarted) return null;

  const send = () => {
    if (!msg.trim()) return;
    sendChat(msg.trim());
    setMsg("");
  };

  return (
    <>
      {/* Toggle button */}
      <button
        onClick={() => setOpen(!open)}
        className="fixed bottom-4 right-4 z-40 w-12 h-12 rounded-full bg-amber-600 text-white shadow-lg
                   hover:bg-amber-700 transition-colors flex items-center justify-center text-xl"
        title="Chat"
      >
        {open ? "✕" : "💬"}
      </button>

      {/* Chat panel */}
      {open && (
        <div className="fixed bottom-20 right-4 z-40 w-80 h-96 bg-white dark:bg-stone-800 rounded-xl shadow-2xl border border-stone-200 dark:border-stone-700 flex flex-col overflow-hidden">
          <div className="px-4 py-2 bg-stone-100 dark:bg-stone-700 border-b border-stone-200 dark:border-stone-600">
            <p className="text-sm font-semibold text-stone-700 dark:text-stone-200">Chat</p>
          </div>

          <div ref={listRef} className="flex-1 overflow-y-auto px-3 py-2 space-y-1.5">
            {chatMessages.length === 0 && (
              <p className="text-xs text-stone-400 text-center mt-4">No messages yet</p>
            )}
            {chatMessages.map((m, i) => (
              <div key={i} className="text-sm">
                <span className="font-semibold text-amber-700 dark:text-amber-400">{m.playerName}</span>
                <span className="text-stone-600 dark:text-stone-400">: {m.message}</span>
              </div>
            ))}
          </div>

          <div className="px-3 py-2 border-t border-stone-200 dark:border-stone-600 flex gap-2">
            <input
              value={msg}
              onChange={(e) => setMsg(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") send(); }}
              placeholder="Type a message..."
              className="flex-1 px-3 py-1.5 text-sm rounded-lg border border-stone-300 dark:border-stone-600
                         bg-white dark:bg-stone-700 text-stone-800 dark:text-stone-200
                         placeholder:text-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
            />
            <button
              onClick={send}
              disabled={!msg.trim()}
              className="px-3 py-1.5 rounded-lg bg-amber-600 text-white text-sm font-medium
                         hover:bg-amber-700 disabled:opacity-40 transition-colors"
            >
              Send
            </button>
          </div>
        </div>
      )}
    </>
  );
}
