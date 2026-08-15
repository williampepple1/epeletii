"use client";

import React, { useState } from "react";
import { useGameStore } from "@/store/gameStore";

export function AuthForm() {
  const [isSignUp, setIsSignUp] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [displayName, setDisplayName] = useState("");
  const authLoading = useGameStore((s) => s.authLoading);
  const authError = useGameStore((s) => s.authError);
  const isLoggedIn = useGameStore((s) => s.isLoggedIn);

  const signUp = useGameStore((s) => s.signUp);
  const signIn = useGameStore((s) => s.signIn);

  if (isLoggedIn) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (isSignUp) {
      signUp(email, password, displayName);
    } else {
      signIn(email, password);
    }
  };

  return (
    <div className="max-w-md mx-auto bg-[var(--surface)] rounded-xl shadow-lg p-6 space-y-4">
      <div className="flex justify-center mb-2">
        <div className="w-20 h-20 rounded-2xl bg-amber-500/10 border-2 border-amber-500/20 flex items-center justify-center text-4xl shadow-sm">
          🦛
        </div>
      </div>
      <h2 className="text-2xl font-bold text-center text-[var(--foreground)]">
        🦛 Epeletii
      </h2>
      <p className="text-center text-[var(--muted)] text-sm">
        {isSignUp ? "Create an account" : "Sign in to play"}
      </p>

      {authError && (
        <div className="bg-red-100 border border-red-300 text-red-700 px-4 py-2 rounded-lg text-sm">
          {authError}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-3">
        <input
          type="email"
          placeholder="Email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
          className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                     focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
        />
        <div className="relative">
          <input
            type={showPassword ? "text" : "password"}
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            minLength={6}
            className="w-full px-4 py-2 pr-11 border border-stone-300 rounded-lg text-stone-800
                       focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            aria-label={showPassword ? "Hide password" : "Show password"}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-stone-400 hover:text-amber-600 transition-colors focus:outline-none"
          >
            {showPassword ? (
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"/>
                <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            ) : (
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            )}
          </button>
        </div>
        {isSignUp && (
          <input
            type="text"
            placeholder="Display name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            required
            className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                       focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
          />
        )}
        <button
          type="submit"
          disabled={authLoading}
          className="w-full py-2 bg-amber-600 text-white rounded-lg font-semibold
                     hover:bg-amber-700 disabled:opacity-40 transition-colors"
        >
          {authLoading ? "Please wait..." : isSignUp ? "Sign Up" : "Sign In"}
        </button>
      </form>

      <button
        onClick={() => setIsSignUp(!isSignUp)}
        className="w-full text-sm text-amber-700 hover:text-amber-800 text-center"
      >
        {isSignUp ? "Already have an account? Sign In" : "No account? Sign Up"}
      </button>
    </div>
  );
}
