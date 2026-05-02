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

  const login = useCallback(() => {
    const base = serverUrl ?? getBaseUrl();
    if (IS_TAURI) {
      // Open SSO in the system browser so WebKit is not involved in the OAuth
      // redirect chain. WebKit rejects HTTP 302 redirects to non-HTTP schemes
      // (e.g. tauri://) with "Redirection to URL with a scheme that is not
      // HTTP(S)". Using the system browser avoids this entirely.
      //
      // The server redirects to ex://app/oidc/callback?token=... after auth.
      // The OS delivers this as a deep link, useDeepLink() navigates to
      // /oidc/callback?token=..., and OIDCCallbackPage completes sign-in.
      const redirectTo = 'ex://app/oidc/callback';
      import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
        openUrl(`${base}/auth/oidc/login?redirect_to=${encodeURIComponent(redirectTo)}`);
      });
    } else {
      window.location.href = `${base}/auth/oidc/login`;
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
