import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { IS_TAURI } from '@/platform';
import { toast } from 'sonner';

export function useDeepLink() {
  const navigate = useNavigate();

  useEffect(() => {
    if (!IS_TAURI) return;

    function handleUrl(rawUrl: string, source: string) {
      toast.info(`[dbg] deep-link(${source}): ${rawUrl.slice(0, 80)}`);
      try {
        const url = new URL(rawUrl);
        const path = url.pathname + url.search + url.hash;
        if (path && path !== '/') navigate(path, { replace: true });
      } catch {
        toast.error(`[dbg] bad URL: ${rawUrl.slice(0, 40)}`);
      }
    }

    let cleanupPlugin: (() => void) | undefined;
    let cleanupFallback: (() => void) | undefined;

    import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl, getCurrent }) => {
      // Check if the app was launched via deep link (cold-start or new instance).
      getCurrent().then((urls) => {
        if (urls) {
          toast.info(`[dbg] getCurrent: ${JSON.stringify(urls).slice(0, 80)}`);
          for (const url of urls) handleUrl(url, 'getCurrent');
        } else {
          toast.info('[dbg] getCurrent: null');
        }
      }).catch((e) => toast.error(`[dbg] getCurrent error: ${e}`));

      // Listen for future deep links (single-instance forwarding on Linux/Windows,
      // NSAppleEvent on macOS).
      onOpenUrl((urls) => {
        for (const url of urls) handleUrl(url, 'onOpenUrl');
      }).then((unlisten) => { cleanupPlugin = unlisten; });
    }).catch((e) => toast.error(`[dbg] plugin import error: ${e}`));

    // Fallback: custom Rust event emitted from single-instance callback
    // and on_open_url handler.
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<string>('deep-link', (event) => {
        handleUrl(event.payload, 'rustEvent');
      }).then((unlisten) => { cleanupFallback = unlisten; });
    });

    return () => {
      cleanupPlugin?.();
      cleanupFallback?.();
    };
  }, [navigate]);
}
