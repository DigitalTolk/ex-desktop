import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { IS_TAURI } from '@/platform';

/**
 * Listens for ex:// deep-link events and navigates to the in-app route.
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

    function handleUrl(rawUrl: string) {
      try {
        const url = new URL(rawUrl);
        const path = url.pathname + url.search + url.hash;
        if (path && path !== '/') navigate(path, { replace: true });
      } catch {
        // ignore malformed URLs
      }
    }

    let cleanupPlugin: (() => void) | undefined;
    let cleanupFallback: (() => void) | undefined;

    // Primary: use the official plugin JS API so we receive URLs delivered
    // via both cold-launch args and the single-instance forwarding mechanism.
    import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
      onOpenUrl((urls) => {
        for (const url of urls) handleUrl(url);
      }).then((unlisten) => { cleanupPlugin = unlisten; });
    }).catch(() => {
      // Plugin JS not available (shouldn't happen in production).
    });

    // Fallback: our custom Rust backend also emits a 'deep-link' event from
    // the single-instance callback and on_open_url handler. Handle both so
    // either mechanism is sufficient.
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<string>('deep-link', (event) => {
        handleUrl(event.payload);
      }).then((unlisten) => { cleanupFallback = unlisten; });
    });

    return () => {
      cleanupPlugin?.();
      cleanupFallback?.();
    };
  }, [navigate]);
}
