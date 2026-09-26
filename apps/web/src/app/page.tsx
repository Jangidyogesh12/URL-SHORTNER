"use client";

import { useEffect, useState } from "react";
import type { HelloResponse, User } from "shared-types";
import { getHello, getUsers } from "@/lib/api";

export default function Home() {
  const [hello, setHello] = useState<HelloResponse | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getHello()
      .then(setHello)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : "unknown error"),
      );
    getUsers().then(setUsers).catch(() => setUsers([]));
  }, []);

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-6 bg-zinc-50 p-8 font-sans dark:bg-black">
      <h1 className="text-3xl font-semibold text-black dark:text-zinc-50">
        Rust + Next.js Monorepo
      </h1>

      <section className="w-full max-w-md rounded-lg border border-black/[.08] p-6 dark:border-white/[.145]">
        <h2 className="mb-2 text-sm font-medium uppercase tracking-wide text-zinc-500">
          GET /api/hello
        </h2>
        {error && <p className="text-red-500">{error}</p>}
        {hello && (
          <pre className="overflow-auto text-sm text-zinc-800 dark:text-zinc-200">
            {JSON.stringify(hello, null, 2)}
          </pre>
        )}
      </section>

      <section className="w-full max-w-md rounded-lg border border-black/[.08] p-6 dark:border-white/[.145]">
        <h2 className="mb-2 text-sm font-medium uppercase tracking-wide text-zinc-500">
          GET /api/users
        </h2>
        {users.length === 0 ? (
          <p className="text-sm text-zinc-500">
            No users yet (database not seeded).
          </p>
        ) : (
          <ul className="text-sm text-zinc-800 dark:text-zinc-200">
            {users.map((user) => (
              <li key={user.id}>
                {user.id} — {user.name} ({user.email})
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
