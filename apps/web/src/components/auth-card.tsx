"use client";

import { useState, type FormEvent, type InputHTMLAttributes } from "react";
import { useAuth } from "@/components/auth-provider";

type AuthMode = "login" | "register";

interface FieldProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "className"
> {
  label: string;
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const PHONE_PATTERN = /^[+0-9][0-9\s().-]{5,}$/;

function Field({ label, ...props }: FieldProps) {
  return (
    <label className="block text-left">
      <span className="mb-1 block text-sm font-medium text-zinc-700 dark:text-zinc-200">
        {label}
      </span>
      <input
        {...props}
        className="w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 dark:border-zinc-700 dark:bg-zinc-950 dark:text-zinc-100 dark:focus:border-zinc-100"
      />
    </label>
  );
}

export default function AuthCard() {
  const { busy, error, login, register } = useAuth();
  const [mode, setMode] = useState<AuthMode>("login");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [formError, setFormError] = useState<string | null>(null);

  function switchMode(nextMode: AuthMode) {
    setMode(nextMode);
    setFormError(null);
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedEmail = email.trim();

    if (!EMAIL_PATTERN.test(normalizedEmail)) {
      setFormError("Enter a valid email address.");
      return;
    }

    if (password.length < 8) {
      setFormError("Use a password with at least 8 characters.");
      return;
    }

    if (mode === "login") {
      setFormError(null);
      await login({ email: normalizedEmail, password });
      return;
    }

    if (name.trim().length < 2) {
      setFormError("Enter the name for the new account.");
      return;
    }

    const normalizedPhone = phone.trim();
    if (normalizedPhone && !PHONE_PATTERN.test(normalizedPhone)) {
      setFormError("Enter a valid phone number or leave it blank.");
      return;
    }

    if (password !== confirmPassword) {
      setFormError("Passwords do not match.");
      return;
    }

    setFormError(null);
    await register({
      name: name.trim(),
      email: normalizedEmail,
      phone: normalizedPhone || null,
      password,
    });
  }

  const visibleError = formError ?? error;

  return (
    <section
      aria-label={mode === "login" ? "Sign in" : "Create an account"}
      className="w-full max-w-md rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-950"
    >
      <div
        role="tablist"
        aria-label="Authentication choices"
        className="mb-6 grid grid-cols-2 gap-1 rounded-xl bg-zinc-100 p-1 dark:bg-zinc-900"
      >
        {(["login", "register"] as AuthMode[]).map((choice) => (
          <button
            key={choice}
            type="button"
            role="tab"
            aria-selected={mode === choice}
            disabled={busy}
            onClick={() => switchMode(choice)}
            className={`rounded-lg px-3 py-2 text-sm font-medium transition ${
              mode === choice
                ? "bg-white text-zinc-900 shadow dark:bg-zinc-950 dark:text-zinc-50"
                : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-100"
            }`}
          >
            {choice === "login" ? "Sign in" : "Create account"}
          </button>
        ))}
      </div>

      <h2 className="text-xl font-semibold text-zinc-900 dark:text-zinc-50">
        {mode === "login" ? "Welcome back" : "Create your link account"}
      </h2>
      <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
        {mode === "login"
          ? "Sign in to see every short link connected to your long URLs."
          : "Register once, and the app signs you in automatically."}
      </p>

      <form onSubmit={handleSubmit} className="mt-5 space-y-4">
        {mode === "register" && (
          <>
            <Field
              label="Name"
              id="register-name"
              name="name"
              type="text"
              autoComplete="name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
              minLength={2}
              disabled={busy}
            />
            <Field
              label="Phone (Optional)"
              id="register-phone"
              name="phone"
              type="tel"
              autoComplete="tel"
              placeholder="+1 555 010 2030"
              value={phone}
              onChange={(event) => setPhone(event.target.value)}
              disabled={busy}
            />
          </>
        )}

        <Field
          label="Email"
          id={`${mode}-email`}
          name="email"
          type="email"
          autoComplete="email"
          value={email}
          onChange={(event) => setEmail(event.target.value)}
          required
          disabled={busy}
        />
        <Field
          label="Password"
          id={`${mode}-password`}
          name="password"
          type="password"
          autoComplete={mode === "login" ? "current-password" : "new-password"}
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          required
          minLength={8}
          disabled={busy}
        />

        {mode === "register" && (
          <Field
            label="Confirm password"
            id="register-confirm-password"
            name="confirmPassword"
            type="password"
            autoComplete="new-password"
            value={confirmPassword}
            onChange={(event) => setConfirmPassword(event.target.value)}
            required
            minLength={8}
            disabled={busy}
          />
        )}

        {visibleError && (
          <p role="alert" className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700 dark:bg-red-950/60 dark:text-red-200">
            {visibleError}
          </p>
        )}

        <button
          type="submit"
          disabled={busy}
          className="w-full rounded-lg bg-zinc-900 px-3 py-2.5 text-sm font-semibold text-white transition hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
        >
          {busy
            ? mode === "login"
              ? "Signing in…"
              : "Creating account…"
            : mode === "login"
              ? "Sign in"
              : "Create account and sign in"}
        </button>
      </form>
    </section>
  );
}
