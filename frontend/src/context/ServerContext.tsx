import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from 'react';
import { IS_TAURI, getServerUrl, setServerUrl } from '@/platform';
import { setBaseUrl } from '@/lib/api';

interface ServerState {
  /** Configured server URL, or null if not yet set up. */
  serverUrl: string | null;
  isLoading: boolean;
  setServer: (url: string) => Promise<void>;
  clearServer: () => Promise<void>;
}

const ServerContext = createContext<ServerState | undefined>(undefined);

function applyBaseUrl(url: string) {
  // In Vite dev mode the proxy already handles routing — setting a base URL
  // would bypass it and cause CORS errors. Only apply in production builds.
  if (!import.meta.env.DEV) {
    setBaseUrl(url);
  }
}

export function ServerProvider({ children }: { children: ReactNode }) {
  // In web mode there is no server selection — the page origin IS the server.
  const [serverUrl, setServerUrlState] = useState<string | null>(IS_TAURI ? null : '');
  const [isLoading, setIsLoading] = useState(IS_TAURI);

  useEffect(() => {
    if (!IS_TAURI) return;
    getServerUrl().then((url) => {
      if (url) {
        setServerUrlState(url);
        applyBaseUrl(url);
      }
      setIsLoading(false);
    });
  }, []);

  const setServer = useCallback(async (url: string) => {
    const normalized = url.replace(/\/$/, '');
    await setServerUrl(normalized);
    setServerUrlState(normalized);
    applyBaseUrl(normalized);
  }, []);

  const clearServer = useCallback(async () => {
    await setServerUrl('');
    setServerUrlState(null);
    setBaseUrl('');
    try { localStorage.removeItem('ex_server_url'); } catch { /* ignore */ }
  }, []);

  return (
    <ServerContext.Provider value={{ serverUrl, isLoading, setServer, clearServer }}>
      {children}
    </ServerContext.Provider>
  );
}

export function useServer(): ServerState {
  const ctx = useContext(ServerContext);
  if (!ctx) throw new Error('useServer must be used inside <ServerProvider>');
  return ctx;
}
