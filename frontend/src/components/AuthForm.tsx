"use client";

import React, { useState } from "react";
import { useGameStore } from "@/store/gameStore";

export function AuthForm() {
  const [isSignUp, setIsSignUp] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
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
    <div className="max-w-sm mx-auto bg-white rounded-xl shadow-lg p-6 space-y-4">
      <h2 className="text-2xl font-bold text-center text-stone-800">
        ⚕ Epeletii
      </h2>
      <p className="text-center text-stone-500 text-sm">
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
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          minLength={6}
          className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                     focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-800"
        />
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
