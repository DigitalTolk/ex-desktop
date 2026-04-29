import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react';
import type { User } from '@/types';
import {
  setAccessToken,
  clearAccessToken,
  apiFetch,
  baseFetch,
  getBaseUrl,
} from '@/lib/api';
import { IS_TAURI, getRefreshToken, setRefreshToken, deleteRefreshToken } from '@/platform';
import { useServer } from '@/context/ServerContext';

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: () => void;
  logout: () => Promise<void>;
  setAuth: (token: string, user: User) => void;
}

const AuthContext = createContext<AuthState | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const { serverUrl } = useServer();
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const isAuthenticated = !!user;

  useEffect(() => {
    async function tryRestore() {
      try {
        // In Tauri, try the keychain refresh token first so sessions survive
        // cookie loss (e.g. after OS reboot clears session cookies).
        if (IS_TAURI) {
          const storedToken = await getRefreshToken();
          if (storedToken) {
            const res = await baseFetch('/auth/token/refresh', {
              method: 'POST',
              credentials: 'include',
              headers: { 'X-Refresh-Token': storedToken },
            });
            if (res.ok) {
              const data = await res.json();
              if (data.accessToken) {
                setAccessToken(data.accessToken);
                const me = await apiFetch<User>('/api/v1/users/me');
                setUser(me);
                setIsLoading(false);
                return;
              }
            }
          }
        }
        // Fall back to httpOnly cookie (web and Tauri production with active session)
        const res = await baseFetch('/auth/token/refresh', {
          method: 'POST',
          credentials: 'include',
        });
        if (res.ok) {
          const data = await res.json();
          if (data.accessToken) {
            setAccessToken(data.accessToken);
            const me = await apiFetch<User>('/api/v1/users/me');
            setUser(me);
          }
        }
      } catch {
        // not authenticated
      } finally {
        setIsLoading(false);
      }
    }
    tryRestore();
  }, []);

  const login = useCallback(() => {
    if (IS_TAURI) {
      // SSO must navigate directly to the server — NOT through the Vite proxy.
      // The OIDC provider redirects back to the real server callback URL, so
      // cookies set via the proxy (bound to localhost:5173) would be missing.
      // serverUrl is always set by the time login is reachable.
      const base = serverUrl ?? getBaseUrl();
      const redirectTo = import.meta.env.DEV
        ? 'http://localhost:5173/oidc/callback'
        : 'tauri://localhost/oidc/callback';
      window.location.href = `${base}/auth/oidc/login?redirect_to=${encodeURIComponent(redirectTo)}`;
    } else {
      window.location.href = `${getBaseUrl()}/auth/oidc/login`;
    }
  }, [serverUrl]);

  const logout = useCallback(async () => {
    try {
      await baseFetch('/auth/logout', {
        method: 'POST',
        credentials: 'include',
      });
    } catch {
      // ignore
    }
    clearAccessToken();
    setUser(null);
    await deleteRefreshToken();
  }, []);

  const setAuth = useCallback((token: string, userData: User) => {
    setAccessToken(token);
    setUser(userData);
  }, []);

  return (
    <AuthContext.Provider
      value={{ user, isAuthenticated, isLoading, login, logout, setAuth }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return ctx;
}
