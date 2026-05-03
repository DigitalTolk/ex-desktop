import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { IS_TAURI } from '@/platform';

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

    import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl, getCurrent }) => {
      getCurrent().then((urls) => {
        if (urls) for (const url of urls) handleUrl(url);
      }).catch(() => {});

      onOpenUrl((urls) => {
        for (const url of urls) handleUrl(url);
      }).then((unlisten) => { cleanupPlugin = unlisten; });
    }).catch(() => {});

    // Fallback: custom Rust event emitted from single-instance callback
    // and on_open_url handler.
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
