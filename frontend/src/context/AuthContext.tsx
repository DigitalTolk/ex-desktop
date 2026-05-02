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
import { IS_TAURI, getRefreshToken, deleteRefreshToken } from '@/platform';
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

  const setAuth = useCallback((token: string, userData: User) => {
    setAccessToken(token);
    setUser(userData);
  }, []);

  const login = useCallback(() => {
    const base = serverUrl ?? getBaseUrl();
    if (IS_TAURI) {
      // Start a one-shot local HTTP server, open SSO in the system browser
      // with redirect_to=http://localhost:{port}/callback. The server hits
      // that endpoint after auth and we emit 'oauth-token' with the token.
      // More reliable than custom URL schemes: no OS registration needed,
      // works in all browsers, no single-instance complexity.
      Promise.all([
        import('@tauri-apps/api/core'),
        import('@tauri-apps/api/event'),
        import('@tauri-apps/plugin-opener'),
      ]).then(async ([{ invoke }, { listen }, { openUrl }]) => {
        const port = await invoke<number>('start_oauth_server');
        const redirectTo = `http://localhost:${port}/callback`;

        const unlisten = await listen<string>('oauth-token', async (event) => {
          unlisten();
          const token = event.payload;
          setAccessToken(token);
          try {
            const user = await apiFetch<User>('/api/v1/users/me');
            setAuth(token, user);
          } catch {
            // token arrived but /users/me failed — stay on login
          }
        });

        openUrl(`${base}/auth/oidc/login?redirect_to=${encodeURIComponent(redirectTo)}`);
      });
    } else {
      window.location.href = `${base}/auth/oidc/login`;
    }
  }, [serverUrl, setAuth]);

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
