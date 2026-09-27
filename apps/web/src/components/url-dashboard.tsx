"use client";

import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import type { UrlReadDto } from "shared-types";
import {
  createUrl,
  deleteUrl,
  getErrorMessage,
  isUnauthorized,
  listUrls,
  updateUrl,
} from "@/lib/api";
import type { AuthSession } from "@/lib/session";

type StatusFilter = "all" | "active" | "expired";

const LINK_LIFETIME_OPTIONS = [
  { label: "30 minutes", value: "30" },
  { label: "2 hours", value: "120" },
  { label: "1 week", value: "10080" },
  { label: "1 year", value: "525600" },
  { label: "No expiry", value: "unlimited" },
] as const;

type LinkLifetimeValue =
  | (typeof LINK_LIFETIME_OPTIONS)[number]["value"]
  | "";

interface UrlDashboardProps {
  session: AuthSession;
  onUnauthorized: () => void;
}

function normalizeDestinationUrl(rawValue: string): string {
  const trimmedValue = rawValue.trim();
  if (!trimmedValue) {
    throw new Error("Enter a destination URL.");
  }

  const candidate = /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmedValue)
    ? trimmedValue
    : `https://${trimmedValue}`;

  let parsed: URL;
  try {
    parsed = new URL(candidate);
  } catch {
    throw new Error("Enter a valid URL, for example https://example.com/page.");
  }

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("Only http and https destinations can be shortened.");
  }

  return parsed.toString();
}

function isExpired(expiresAt: string | null, now: number = Date.now()): boolean {
  if (!expiresAt) {
    return false;
  }

  const timestamp = Date.parse(expiresAt);
  return !Number.isNaN(timestamp) && timestamp <= now;
}

function formatExpiry(value: string | null): string {
  if (!value) {
    return "No expiry";
  }

  return formatDateTime(value);
}

function formatDateTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export default function UrlDashboard({
  session,
  onUnauthorized,
}: UrlDashboardProps) {
  const [urls, setUrls] = useState<UrlReadDto[]>([]);
  const [loadingList, setLoadingList] = useState(true);
  const [listError, setListError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [destination, setDestination] = useState("");
  const [saving, setSaving] = useState(false);
  const [editingCode, setEditingCode] = useState<string | null>(null);
  const [draftDestination, setDraftDestination] = useState("");
  const [draftLifetime, setDraftLifetime] = useState<LinkLifetimeValue>("");
  const [pendingDeleteCode, setPendingDeleteCode] = useState<string | null>(
    null,
  );
  const [deleting, setDeleting] = useState(false);
  const [copiedCode, setCopiedCode] = useState<string | null>(null);
  const [origin, setOrigin] = useState<string | null>(null);
  const requestCounter = useRef(0);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Read the browser origin after mount.
    setOrigin(window.location.origin);
  }, []);

  const shortHref = useCallback(
    (shortCode: string) =>
      origin ? new URL(`/${shortCode}`, origin).toString() : `/${shortCode}`,
    [origin],
  );

  const loadUrls = useCallback(async () => {
    const requestId = requestCounter.current + 1;
    requestCounter.current = requestId;
    setLoadingList(true);
    setListError(null);

    try {
      const data = await listUrls(session.token);
      if (requestCounter.current !== requestId) {
        return;
      }
      setUrls(data);
    } catch (error) {
      if (requestCounter.current !== requestId) {
        return;
      }
      if (isUnauthorized(error)) {
        onUnauthorized();
        return;
      }
      setListError(getErrorMessage(error, "Unable to load short links."));
    } finally {
      if (requestCounter.current === requestId) {
        setLoadingList(false);
      }
    }
  }, [session.token, onUnauthorized]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Fetch the authenticated URL list when the session changes.
    void loadUrls();
  }, [loadUrls]);

  const stats = useMemo(() => {
    let active = 0;
    let expired = 0;

    for (const url of urls) {
      if (isExpired(url.expires_at)) {
        expired += 1;
      } else {
        active += 1;
      }
    }

    return { total: urls.length, active, expired };
  }, [urls]);

  const filteredUrls = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();

    return urls.filter((url) => {
      const expired = isExpired(url.expires_at);
      if (statusFilter === "active" && expired) {
        return false;
      }
      if (statusFilter === "expired" && !expired) {
        return false;
      }

      if (!normalizedQuery) {
        return true;
      }

      return (
        url.long_url.toLowerCase().includes(normalizedQuery) ||
        url.short_code.toLowerCase().includes(normalizedQuery)
      );
    });
  }, [urls, query, statusFilter]);

  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setActionError(null);
    setNotice(null);

    let normalized: string;
    try {
      normalized = normalizeDestinationUrl(destination);
    } catch (error) {
      setActionError(getErrorMessage(error, "Enter a valid destination URL."));
      return;
    }

    setSaving(true);
    try {
      const created = await createUrl({ long_url: normalized }, session.token);
      setUrls((current) => [
        created,
        ...current.filter((url) => url.id !== created.id),
      ]);
      setDestination("");
      setNotice(`Short link ready: ${shortHref(created.short_code)}`);
    } catch (error) {
      if (isUnauthorized(error)) {
        onUnauthorized();
        return;
      }
      setActionError(getErrorMessage(error, "Unable to create a short link."));
    } finally {
      setSaving(false);
    }
  }

  function startEditing(url: UrlReadDto) {
    setEditingCode(url.short_code);
    setDraftDestination(url.long_url);
    setDraftLifetime("");
    setPendingDeleteCode(null);
    setActionError(null);
    setNotice(null);
  }

  function cancelEditing() {
    setEditingCode(null);
    setDraftDestination("");
    setDraftLifetime("");
  }

  async function handleUpdate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!editingCode) {
      return;
    }

    setActionError(null);
    setNotice(null);

    let normalized: string;
    try {
      normalized = normalizeDestinationUrl(draftDestination);
    } catch (error) {
      setActionError(getErrorMessage(error, "Enter a valid destination URL."));
      return;
    }

    const selectedLifetime = LINK_LIFETIME_OPTIONS.find(
      (option) => option.value === draftLifetime,
    );
    if (!selectedLifetime) {
      setActionError("Select a link lifetime.");
      return;
    }

    setSaving(true);
    try {
      const updated = await updateUrl(
        {
          short_code: editingCode,
          new_url: normalized,
          expires_in_minutes:
            selectedLifetime.value === "unlimited"
              ? null
              : Number(selectedLifetime.value),
        },
        session.token,
      );
      setUrls((current) =>
        current.map((url) =>
          url.short_code === updated.short_code ? updated : url,
        ),
      );
      setEditingCode(null);
      setDraftDestination("");
      setDraftLifetime("");
      setNotice(
        `Updated destination for ${updated.short_code}. New expiry: ${formatExpiry(updated.expires_at)}.`,
      );
    } catch (error) {
      if (isUnauthorized(error)) {
        onUnauthorized();
        return;
      }
      setActionError(getErrorMessage(error, "Unable to update the short link."));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(shortCode: string) {
    const target = urls.find((url) => url.short_code === shortCode);
    if (!target) {
      return;
    }

    setActionError(null);
    setNotice(null);
    setDeleting(true);
    try {
      await deleteUrl({ long_url: target.long_url }, session.token);
      setUrls((current) =>
        current.filter((url) => url.short_code !== shortCode),
      );
      setPendingDeleteCode(null);
      setNotice(`Deleted ${shortCode}.`);
    } catch (error) {
      if (isUnauthorized(error)) {
        onUnauthorized();
        return;
      }
      setActionError(getErrorMessage(error, "Unable to delete the short link."));
    } finally {
      setDeleting(false);
    }
  }

  async function handleCopy(url: UrlReadDto) {
    const text = shortHref(url.short_code);

    try {
      await navigator.clipboard.writeText(text);
      setCopiedCode(url.short_code);
      setActionError(null);
    } catch {
      setActionError("This browser blocked access to the clipboard.");
    }
  }

  return (
    <div className="w-full max-w-4xl space-y-6">
      <section className="rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-950">
        <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-50">
          Create a short link
        </h2>
        <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
          Short links live at this domain. Anyone opening one is redirected to
          the original website.
        </p>

        <form onSubmit={handleCreate} className="mt-4 flex flex-col gap-3 sm:flex-row">
          <label htmlFor="new-long-url" className="sr-only">
            Destination URL
          </label>
          <input
            id="new-long-url"
            name="longUrl"
            type="url"
            inputMode="url"
            autoComplete="url"
            placeholder="https://example.com/very/long/page"
            value={destination}
            onChange={(event) => setDestination(event.target.value)}
            disabled={saving}
            required
            className="flex-1 rounded-lg border border-zinc-300 bg-white px-3 py-2.5 text-sm text-zinc-900 shadow-sm outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-100"
          />
          <button
            type="submit"
            disabled={saving}
            className="rounded-lg bg-zinc-900 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
          >
            {saving ? "Shortening…" : "Shorten URL"}
          </button>
        </form>
      </section>

      <section
        aria-live="polite"
        className="rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-950"
      >
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-50">
              Your short links
            </h2>
            <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
              {stats.total} total · {stats.active} active · {stats.expired}{" "}
              expired
            </p>
          </div>

          <div className="flex flex-col gap-3 sm:flex-row">
            <label className="block">
              <span className="sr-only">Search short links</span>
              <input
                type="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search long URL or code"
                className="w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 sm:w-64 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-100"
              />
            </label>
            <label className="block">
              <span className="sr-only">Filter by link status</span>
              <select
                value={statusFilter}
                onChange={(event) =>
                  setStatusFilter(event.target.value as StatusFilter)
                }
                className="w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 sm:w-auto dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-100"
              >
                <option value="all">All statuses</option>
                <option value="active">Active only</option>
                <option value="expired">Expired only</option>
              </select>
            </label>
            <button
              type="button"
              onClick={() => void loadUrls()}
              disabled={loadingList}
              className="rounded-lg border border-zinc-300 px-3 py-2 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-60 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
            >
              {loadingList ? "Refreshing…" : "Refresh"}
            </button>
          </div>
        </div>

        {notice && (
          <p className="mt-4 rounded-lg bg-emerald-50 px-3 py-2 text-sm break-words text-emerald-800 dark:bg-emerald-950/60 dark:text-emerald-100">
            {notice}
          </p>
        )}
        {actionError && (
          <p
            role="alert"
            className="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700 dark:bg-red-950/60 dark:text-red-200"
          >
            {actionError}
          </p>
        )}
        {listError && (
          <p
            role="alert"
            className="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700 dark:bg-red-950/60 dark:text-red-200"
          >
            {listError}
          </p>
        )}

        {loadingList ? (
          <p className="mt-6 text-sm text-zinc-500 dark:text-zinc-400">
            Loading your links…
          </p>
        ) : filteredUrls.length === 0 ? (
          <p className="mt-6 rounded-lg bg-zinc-50 px-4 py-6 text-center text-sm text-zinc-500 dark:bg-zinc-900 dark:text-zinc-400">
            {urls.length === 0
              ? "No short links yet. Create your first link above."
              : "No links match this search or status filter."}
          </p>
        ) : (
          <ul className="mt-6 space-y-4">
            {filteredUrls.map((url) => {
              const expired = isExpired(url.expires_at);
              const href = shortHref(url.short_code);
              const isEditing = editingCode === url.short_code;
              const confirmingDelete = pendingDeleteCode === url.short_code;

              return (
                <li
                  key={url.id}
                  className="rounded-xl border border-zinc-200 p-4 dark:border-zinc-800"
                >
                  <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                    <div className="min-w-0">
                      <a
                        href={url.long_url}
                        target="_blank"
                        rel="noreferrer"
                        className="block truncate text-sm font-medium text-zinc-900 underline decoration-zinc-300 underline-offset-4 hover:decoration-zinc-700 dark:text-zinc-50"
                        title={url.long_url}
                      >
                        {url.long_url}
                      </a>
                      <a
                        href={href}
                        target="_blank"
                        rel="noreferrer"
                        className="mt-1 block truncate font-mono text-sm text-emerald-700 hover:underline dark:text-emerald-300"
                        title={href}
                      >
                        {href}
                      </a>
                    </div>

                    <span
                      className={`inline-flex w-fit items-center rounded-full px-2.5 py-1 text-xs font-semibold ${
                        expired
                          ? "bg-amber-100 text-amber-800 dark:bg-amber-950/60 dark:text-amber-200"
                          : "bg-emerald-100 text-emerald-800 dark:bg-emerald-950/60 dark:text-emerald-200"
                      }`}
                    >
                      {expired ? "Expired" : "Active"}
                    </span>
                  </div>

                  <dl className="mt-3 grid grid-cols-1 gap-2 text-xs text-zinc-500 sm:grid-cols-3 dark:text-zinc-400">
                    <div>
                      <dt className="font-medium uppercase tracking-wide">
                        Code
                      </dt>
                      <dd className="font-mono text-sm text-zinc-800 dark:text-zinc-200">
                        {url.short_code}
                      </dd>
                    </div>
                    <div>
                      <dt className="font-medium uppercase tracking-wide">
                        Created
                      </dt>
                      <dd>
                        <time dateTime={url.created_at}>
                          {formatDateTime(url.created_at)}
                        </time>
                      </dd>
                    </div>
                    <div>
                      <dt className="font-medium uppercase tracking-wide">
                        Expires
                      </dt>
                      <dd>
                        {url.expires_at ? (
                          <time dateTime={url.expires_at}>
                            {formatExpiry(url.expires_at)}
                          </time>
                        ) : (
                          "No expiry"
                        )}
                      </dd>
                    </div>
                  </dl>

                  {isEditing ? (
                    <form
                      onSubmit={handleUpdate}
                      className="mt-4 grid gap-2 sm:grid-cols-[1fr_12rem_auto]"
                    >
                      <label
                        htmlFor={`edit-${url.short_code}`}
                        className="sr-only"
                      >
                        New destination for {url.short_code}
                      </label>
                      <input
                        id={`edit-${url.short_code}`}
                        type="url"
                        inputMode="url"
                        value={draftDestination}
                        onChange={(event) =>
                          setDraftDestination(event.target.value)
                        }
                        disabled={saving}
                        required
                        className="rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-100"
                      />
                      <label
                        htmlFor={`lifetime-${url.short_code}`}
                        className="sr-only"
                      >
                        Link lifetime for {url.short_code}
                      </label>
                      <select
                        id={`lifetime-${url.short_code}`}
                        value={draftLifetime}
                        onChange={(event) =>
                          setDraftLifetime(
                            event.target.value as LinkLifetimeValue,
                          )
                        }
                        disabled={saving}
                        required
                        className="rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 outline-none transition focus:border-zinc-900 focus:ring-2 focus:ring-zinc-900/20 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-100"
                      >
                        <option value="" disabled>
                          Link lifetime
                        </option>
                        {LINK_LIFETIME_OPTIONS.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </select>
                      <div className="flex gap-2">
                        <button
                          type="submit"
                          disabled={saving}
                          className="rounded-lg bg-zinc-900 px-3 py-2 text-sm font-semibold text-white transition hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
                        >
                          Save
                        </button>
                        <button
                          type="button"
                          onClick={cancelEditing}
                          disabled={saving}
                          className="rounded-lg border border-zinc-300 px-3 py-2 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-60 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
                        >
                          Cancel
                        </button>
                      </div>
                    </form>
                  ) : (
                    <div className="mt-4 flex flex-wrap gap-2">
                      <button
                        type="button"
                        onClick={() => void handleCopy(url)}
                        className="rounded-lg border border-zinc-300 px-3 py-1.5 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
                      >
                        {copiedCode === url.short_code ? "Copied!" : "Copy"}
                      </button>
                      <a
                        href={href}
                        target="_blank"
                        rel="noreferrer"
                        className="rounded-lg border border-zinc-300 px-3 py-1.5 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
                      >
                        Open
                      </a>
                      <button
                        type="button"
                        onClick={() => startEditing(url)}
                        className="rounded-lg border border-zinc-300 px-3 py-1.5 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
                      >
                        Edit
                      </button>
                      {confirmingDelete ? (
                        <>
                          <span className="self-center text-sm text-zinc-600 dark:text-zinc-300">
                            Delete this link?
                          </span>
                          <button
                            type="button"
                            onClick={() => void handleDelete(url.short_code)}
                            disabled={deleting}
                            className="rounded-lg bg-red-600 px-3 py-1.5 text-sm font-semibold text-white transition hover:bg-red-500 disabled:cursor-not-allowed disabled:opacity-60"
                          >
                            {deleting ? "Deleting…" : "Yes, delete"}
                          </button>
                          <button
                            type="button"
                            onClick={() => setPendingDeleteCode(null)}
                            disabled={deleting}
                            className="rounded-lg border border-zinc-300 px-3 py-1.5 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:cursor-not-allowed disabled:opacity-60 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-900"
                          >
                            Keep
                          </button>
                        </>
                      ) : (
                        <button
                          type="button"
                          onClick={() => setPendingDeleteCode(url.short_code)}
                          className="rounded-lg border border-red-200 px-3 py-1.5 text-sm font-medium text-red-700 transition hover:bg-red-50 dark:border-red-900 dark:text-red-200 dark:hover:bg-red-950/60"
                        >
                          Delete
                        </button>
                      )}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </section>
    </div>
  );
}
