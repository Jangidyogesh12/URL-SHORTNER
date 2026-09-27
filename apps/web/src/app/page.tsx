"use client";

import { useCallback } from "react";
import AuthCard from "@/components/auth-card";
import { AuthProvider, useAuth } from "@/components/auth-provider";
import UrlDashboard from "@/components/url-dashboard";

function AuthenticatedHome() {
  const { initializing, logout, session } = useAuth();
  const handleUnauthorized = useCallback(() => {
    logout();
  }, [logout]);

  if (initializing) {
    return (
      <p className="text-sm text-zinc-500 dark:text-zinc-400">
        Checking your session…
      </p>
    );
  }

  if (!session) {
    return (
      <div className="flex w-full max-w-4xl flex-col items-center gap-8">
        <div className="max-w-2xl text-center">
          <h1 className="text-3xl font-semibold text-zinc-900 sm:text-4xl dark:text-zinc-50">
            Short links for your long URLs
          </h1>
          <p className="mt-3 text-sm text-zinc-500 sm:text-base dark:text-zinc-400">
            Create an account or sign back in to manage short links. Every link
            keeps working at this domain as{" "}
            <span className="font-mono">/SHORT_CODE</span>.
          </p>
        </div>

        <AuthCard />

        <dl className="grid w-full max-w-2xl grid-cols-1 gap-3 text-sm sm:grid-cols-3">
          {[
            ["Private by account", "Links belong to the signed-in user."],
            ["Thirty-day links", "New links last 30 days; edits can shorten or remove expiry."],
            ["Instant redirects", "Opening a code sends visitors onward."],
          ].map(([title, detail]) => (
            <div
              key={title}
              className="rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
            >
              <dt className="font-semibold text-zinc-900 dark:text-zinc-50">
                {title}
              </dt>
              <dd className="mt-1 text-zinc-500 dark:text-zinc-400">{detail}</dd>
            </div>
          ))}
        </dl>
      </div>
    );
  }

  const displayName =
    session.user.name?.trim() || session.user.email.split("@")[0];

  return (
    <div className="flex w-full max-w-4xl flex-col items-center gap-6">
      <header className="flex w-full flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p className="text-sm text-zinc-500 dark:text-zinc-400">
            Signed in as {session.user.email}
          </p>
          <h1 className="text-2xl font-semibold text-zinc-900 sm:text-3xl dark:text-zinc-50">
            Welcome, {displayName}
          </h1>
        </div>
        <button
          type="button"
          onClick={logout}
          className="w-fit rounded-lg border border-zinc-300 px-3 py-2 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
        >
          Sign out
        </button>
      </header>

      <UrlDashboard session={session} onUnauthorized={handleUnauthorized} />
    </div>
  );
}

export default function Home() {
  return (
    <AuthProvider>
      <main className="flex flex-1 items-start justify-center bg-zinc-50 px-4 py-10 font-sans sm:px-8 dark:bg-black">
        <AuthenticatedHome />
      </main>
    </AuthProvider>
  );
}
