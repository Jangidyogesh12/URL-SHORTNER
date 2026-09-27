import type { TokenClaimsDto } from "shared-types";

export interface SessionUser {
  id: string;
  email: string;
  name?: string;
}

export interface AuthSession {
  token: string;
  user: SessionUser;
  expiresAt: number | null;
}

interface StoredToken {
  token: string;
}

const SESSION_STORAGE_KEY = "url-shortener.session.v1";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodeBase64UrlSegment(segment: string): string {
  const normalized = segment.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized.padEnd(
    normalized.length + ((4 - (normalized.length % 4)) % 4),
    "=",
  );
  const binary = atob(padded);
  const bytes = Uint8Array.from(binary, (character) =>
    character.charCodeAt(0),
  );

  return new TextDecoder().decode(bytes);
}

function claimsFromToken(token: string): TokenClaimsDto | null {
  const segments = token.split(".");
  if (segments.length !== 3) {
    return null;
  }

  try {
    const claims: unknown = JSON.parse(decodeBase64UrlSegment(segments[1]));
    if (!isRecord(claims)) {
      return null;
    }

    if (
      typeof claims.sub !== "string" ||
      typeof claims.email !== "string" ||
      typeof claims.iat !== "number" ||
      typeof claims.exp !== "number"
    ) {
      return null;
    }

    return {
      sub: claims.sub,
      email: claims.email,
      iat: claims.iat,
      exp: claims.exp,
    };
  } catch {
    return null;
  }
}

export function sessionFromToken(
  token: string,
  fallback?: { email?: string; name?: string },
): AuthSession | null {
  const claims = claimsFromToken(token);
  if (!claims) {
    return null;
  }

  const email = claims.email || fallback?.email;
  if (!email) {
    return null;
  }

  return {
    token,
    user: {
      id: claims.sub,
      email,
      name: fallback?.name,
    },
    expiresAt: Number.isFinite(claims.exp) ? claims.exp * 1000 : null,
  };
}

export function isSessionExpired(
  session: AuthSession,
  now: number = Date.now(),
): boolean {
  return session.expiresAt !== null && session.expiresAt <= now;
}

export function loadStoredSession(): AuthSession | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const raw = window.localStorage.getItem(SESSION_STORAGE_KEY);
    if (!raw) {
      return null;
    }

    const stored: unknown = JSON.parse(raw);
    if (!isRecord(stored) || typeof stored.token !== "string") {
      return null;
    }

    const profile: StoredToken & Record<string, unknown> = {
      ...stored,
      token: stored.token,
    };
    const session = sessionFromToken(profile.token, {
      email: typeof profile.email === "string" ? profile.email : undefined,
      name: typeof profile.name === "string" ? profile.name : undefined,
    });

    if (!session || isSessionExpired(session)) {
      window.localStorage.removeItem(SESSION_STORAGE_KEY);
      return null;
    }

    return session;
  } catch {
    window.localStorage.removeItem(SESSION_STORAGE_KEY);
    return null;
  }
}

export function saveStoredSession(token: string, user: SessionUser): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(
    SESSION_STORAGE_KEY,
    JSON.stringify({
      token,
      email: user.email,
      name: user.name,
    }),
  );
}

export function clearStoredSession(): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.removeItem(SESSION_STORAGE_KEY);
}
