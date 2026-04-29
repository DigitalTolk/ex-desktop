import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { IS_TAURI } from '@/platform';

/**
 * Listens for ex:// deep-link events emitted by the Tauri backend and
 * navigates to the corresponding in-app route.
 *
 * URL format:  ex://<host>/<path>
 * Examples:
 *   ex://app/channel/general       → /channel/general
 *   ex://app/oidc/callback?token=X → /oidc/callback?token=X
 *   ex://app/invite/TOKEN          → /invite/TOKEN
 */
export function useDeepLink() {
  const navigate = useNavigate();

  useEffect(() => {
    if (!IS_TAURI) return;

    let unlisten: (() => void) | undefined;

    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<string>('deep-link', (event) => {
        try {
          const url = new URL(event.payload);
          // Convert ex://app/some/path?q=1 → /some/path?q=1
          const path = url.pathname + url.search + url.hash;
          if (path && path !== '/') navigate(path, { replace: true });
        } catch {
          // ignore malformed URLs
        }
      }).then((fn) => { unlisten = fn; });
    });

    return () => { unlisten?.(); };
  }, [navigate]);
}
