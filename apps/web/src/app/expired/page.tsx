import Link from "next/link";

export default function ExpiredShortLink() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-zinc-50 px-4 py-10 dark:bg-black">
      <div className="w-full max-w-md rounded-2xl border border-zinc-200 bg-white p-8 text-center shadow-sm dark:border-zinc-800 dark:bg-zinc-950">
        <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-50">
          Short link expired
        </h1>
        <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">
          This short link is no longer alive. Its owner can create a fresh short
          link for the destination.
        </p>
        <Link
          href="/"
          className="mt-6 inline-block rounded-lg bg-zinc-900 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-zinc-700 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
        >
          Create a new short link
        </Link>
      </div>
    </main>
  );
}
