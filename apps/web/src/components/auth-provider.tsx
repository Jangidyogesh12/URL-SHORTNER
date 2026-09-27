"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  getErrorMessage,
  loginUser,
  registerUser,
} from "@/lib/api";
import {
  clearStoredSession,
  isSessionExpired,
  loadStoredSession,
  saveStoredSession,
  sessionFromToken,
  type AuthSession,
} from "@/lib/session";

interface LoginInput {
  email: string;
  password: string;
}

interface RegisterInput extends LoginInput {
  name: string;
  phone: string | null;
}

interface AuthContextValue {
  session: AuthSession | null;
  initializing: boolean;
  busy: boolean;
  error: string | null;
  login: (input: LoginInput) => Promise<boolean>;
  register: (input: RegisterInput) => Promise<boolean>;
  logout: () => void;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<AuthSession | null>(null);
  const [initializing, setInitializing] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Load the persisted browser session after mount.
    setSession(loadStoredSession());
    setInitializing(false);
  }, []);

  const logout = useCallback(() => {
    clearStoredSession();
    setSession(null);
    setError(null);
  }, []);

  useEffect(() => {
    if (!session?.expiresAt) {
      return;
    }

    const delay = session.expiresAt - Date.now();
    const timeout = window.setTimeout(logout, Math.max(delay, 0));
    return () => window.clearTimeout(timeout);
  }, [session, logout]);

  const login = useCallback(async (input: LoginInput) => {
    setBusy(true);
    setError(null);

    try {
      const tokenResponse = await loginUser({
        email: input.email.trim(),
        password: input.password,
      });
      const nextSession = sessionFromToken(tokenResponse.token, {
        email: input.email.trim(),
      });

      if (!nextSession || isSessionExpired(nextSession)) {
        setError("The server returned an invalid authentication token.");
        return false;
      }

      saveStoredSession(nextSession.token, nextSession.user);
      setSession(nextSession);
      return true;
    } catch (unknownError) {
      setError(getErrorMessage(unknownError, "Unable to sign in."));
      return false;
    } finally {
      setBusy(false);
    }
  }, []);

  const register = useCallback(
    async (input: RegisterInput) => {
      setBusy(true);
      setError(null);

      try {
        const registeredUser = await registerUser({
          name: input.name.trim(),
          email: input.email.trim(),
          phone: input.phone?.trim() || null,
          password: input.password,
        });
        const signedIn = await login({
          email: input.email.trim(),
          password: input.password,
        });

        if (!signedIn) {
          return false;
        }

        setSession((current) => {
          if (!current) {
            return current;
          }

          const nextSession: AuthSession = {
            ...current,
            user: {
              ...current.user,
              id: registeredUser.id,
              email: registeredUser.email,
              name: registeredUser.name,
            },
          };
          saveStoredSession(nextSession.token, nextSession.user);
          return nextSession;
        });

        return true;
      } catch (unknownError) {
        setError(getErrorMessage(unknownError, "Unable to create an account."));
        return false;
      } finally {
        setBusy(false);
      }
    },
    [login],
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const value = useMemo<AuthContextValue>(
    () => ({
      session,
      initializing,
      busy,
      error,
      login,
      register,
      logout,
      clearError,
    }),
    [session, initializing, busy, error, login, register, logout, clearError],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider.");
  }

  return context;
}
