import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { IS_TAURI } from '@/platform';
import { toast } from 'sonner';

export function useDeepLink() {
  const navigate = useNavigate();

  useEffect(() => {
    if (!IS_TAURI) return;

    function handleUrl(rawUrl: string, source: string) {
      toast.info(`[debug] deep-link (${source}): ${rawUrl.slice(0, 60)}`);
      try {
        const url = new URL(rawUrl);
        const path = url.pathname + url.search + url.hash;
        if (path && path !== '/') navigate(path, { replace: true });
      } catch {
        toast.error(`[debug] malformed deep-link URL`);
      }
    }

    let cleanupPlugin: (() => void) | undefined;
    let cleanupFallback: (() => void) | undefined;

    import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
      onOpenUrl((urls) => {
        for (const url of urls) handleUrl(url, 'plugin');
      }).then((unlisten) => { cleanupPlugin = unlisten; });
    }).catch(() => {});

    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<string>('deep-link', (event) => {
        handleUrl(event.payload, 'event');
      }).then((unlisten) => { cleanupFallback = unlisten; });
    });

    return () => {
      cleanupPlugin?.();
      cleanupFallback?.();
    };
  }, [navigate]);
}
