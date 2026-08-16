"use client";

import React, { useState, useEffect } from "react";
import { useGameStore } from "@/store/gameStore";

export function AuthForm() {
  const [mode, setMode] = useState<"signin" | "signup" | "forgot" | "reset">("signin");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [displayName, setDisplayName] = useState("");
  const [resetToken, setResetToken] = useState("");
  const [newPassword, setNewPassword] = useState("");

  const authLoading = useGameStore((s) => s.authLoading);
  const authError = useGameStore((s) => s.authError);
  const isLoggedIn = useGameStore((s) => s.isLoggedIn);
  const forgotPasswordSuccess = useGameStore((s) => s.forgotPasswordSuccess);
  const resetPasswordSuccess = useGameStore((s) => s.resetPasswordSuccess);

  const signUp = useGameStore((s) => s.signUp);
  const signIn = useGameStore((s) => s.signIn);
  const forgotPassword = useGameStore((s) => s.forgotPassword);
  const resetPassword = useGameStore((s) => s.resetPassword);
  const setForgotPasswordSuccess = useGameStore((s) => s.setForgotPasswordSuccess);
  const setResetPasswordSuccess = useGameStore((s) => s.setResetPasswordSuccess);

  useEffect(() => {
    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      const token = params.get("token");
      const emailParam = params.get("email");
      const action = params.get("action");
      if (token && emailParam && action === "reset-password") {
        setEmail(emailParam);
        setResetToken(token);
        setMode("reset");
        // Clean URL queries so they don't linger
        window.history.replaceState({}, document.title, window.location.pathname);
      }
    }
  }, []);

  if (isLoggedIn) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (mode === "signup") {
      signUp(email, password, displayName);
    } else if (mode === "signin") {
      signIn(email, password);
    } else if (mode === "forgot") {
      forgotPassword(email);
    } else if (mode === "reset") {
      resetPassword(email, resetToken, newPassword);
    }
  };

  const handleSwitchMode = (newMode: "signin" | "signup" | "forgot" | "reset") => {
    setForgotPasswordSuccess(false);
    setResetPasswordSuccess(false);
    setMode(newMode);
  };

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
        {mode === "signin" && "Sign in to play"}
        {mode === "signup" && "Create an account"}
        {mode === "forgot" && "Reset your password"}
        {mode === "reset" && "Choose a new password"}
      </p>

      {authError && (
        <div className="bg-red-100 border border-red-300 text-red-700 px-4 py-2 rounded-lg text-sm">
          {authError}
        </div>
      )}

      {mode === "forgot" && forgotPasswordSuccess ? (
        <div className="space-y-4">
          <div className="bg-green-50 border border-green-300 text-green-800 px-4 py-3 rounded-lg text-sm text-center">
            🔑 Reset link sent! Check your email inbox to reset your password.
          </div>
          <button
            onClick={() => handleSwitchMode("signin")}
            className="w-full py-2 bg-stone-200 dark:bg-stone-700 text-stone-700 dark:text-stone-300 rounded-lg font-semibold hover:bg-stone-300 dark:hover:bg-stone-600 transition-colors"
          >
            Back to Sign In
          </button>
        </div>
      ) : mode === "reset" && resetPasswordSuccess ? (
        <div className="space-y-4">
          <div className="bg-green-50 border border-green-300 text-green-800 px-4 py-3 rounded-lg text-sm text-center">
            ✅ Password reset successfully! You can now sign in.
          </div>
          <button
            onClick={() => handleSwitchMode("signin")}
            className="w-full py-2 bg-amber-600 text-white rounded-lg font-semibold hover:bg-amber-700 transition-colors"
          >
            Go to Sign In
          </button>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-3">
          {mode === "reset" ? (
            <input
              type="email"
              value={email}
              disabled
              className="w-full px-4 py-2 border border-stone-300 rounded-lg bg-stone-100 text-stone-500 cursor-not-allowed font-medium"
            />
          ) : (
            <input
              type="email"
              placeholder="Email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                         focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-400"
            />
          )}

          {mode === "forgot" ? null : mode === "reset" ? (
            <div className="relative">
              <input
                type={showPassword ? "text" : "password"}
                placeholder="New Password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                required
                minLength={6}
                className="w-full px-4 py-2 pr-11 border border-stone-300 rounded-lg text-stone-800
                           focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-400"
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
          ) : (
            <div className="relative">
              <input
                type={showPassword ? "text" : "password"}
                placeholder="Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={6}
                className="w-full px-4 py-2 pr-11 border border-stone-300 rounded-lg text-stone-800
                           focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-400"
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
          )}

          {mode === "signup" && (
            <input
              type="text"
              placeholder="Display name"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              required
              className="w-full px-4 py-2 border border-stone-300 rounded-lg text-stone-800
                         focus:outline-none focus:ring-2 focus:ring-amber-500 placeholder:text-stone-400"
            />
          )}

          <button
            type="submit"
            disabled={authLoading}
            className="w-full py-2 bg-amber-600 text-white rounded-lg font-semibold
                       hover:bg-amber-700 disabled:opacity-40 transition-colors"
          >
            {authLoading
              ? "Please wait..."
              : mode === "signup"
              ? "Sign Up"
              : mode === "signin"
              ? "Sign In"
              : mode === "forgot"
              ? "Send Reset Link"
              : "Reset Password"}
          </button>
        </form>
      )}

      {mode === "signin" && (
        <div className="text-center space-y-2">
          <button
            onClick={() => handleSwitchMode("forgot")}
            className="block w-full text-xs text-stone-500 hover:text-amber-700 text-center transition-colors"
          >
            Forgot your password?
          </button>
          <button
            onClick={() => handleSwitchMode("signup")}
            className="w-full text-sm text-amber-700 hover:text-amber-800 text-center transition-colors"
          >
            No account? Sign Up
          </button>
        </div>
      )}

      {mode === "signup" && (
        <button
          onClick={() => handleSwitchMode("signin")}
          className="w-full text-sm text-amber-700 hover:text-amber-800 text-center transition-colors"
        >
          Already have an account? Sign In
        </button>
      )}

      {(mode === "forgot" || mode === "reset") && !forgotPasswordSuccess && !resetPasswordSuccess && (
        <button
          onClick={() => handleSwitchMode("signin")}
          className="w-full text-sm text-amber-700 hover:text-amber-800 text-center transition-colors"
        >
          Back to Sign In
        </button>
      )}
    </div>
  );
}
