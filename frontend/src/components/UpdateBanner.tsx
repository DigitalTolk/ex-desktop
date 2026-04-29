import { useEffect, useState } from 'react';
import { useServerVersion } from '@/hooks/useServerVersion';
import { Button } from '@/components/ui/button';
import { RefreshCw, Download } from 'lucide-react';
import { IS_TAURI } from '@/platform';

function useNativeUpdateAvailable() {
  const [available, setAvailable] = useState(false);
  useEffect(() => {
    if (!IS_TAURI) return;
    let unlisten: (() => void) | undefined;
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen('update-available', () => setAvailable(true)).then((fn) => { unlisten = fn; });
    });
    return () => { unlisten?.(); };
  }, []);
  return available;
}

// UpdateBanner shows either:
// - A server-deployment update prompt (web and desktop)
// - A native app update prompt (desktop only, from tauri-plugin-updater)
export function UpdateBanner() {
  const { outdated } = useServerVersion();
  const nativeUpdate = useNativeUpdateAvailable();

  if (nativeUpdate) {
    return (
      <div
        role="alert"
        className="flex shrink-0 items-center justify-between gap-3 border-b border-blue-300 bg-blue-100 px-4 py-2 text-sm text-blue-900 dark:border-blue-500/40 dark:bg-blue-500/15 dark:text-blue-100"
      >
        <div className="flex items-center gap-2">
          <Download className="h-4 w-4" aria-hidden="true" />
          <span>A new version of <strong>ex</strong> is available.</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={async () => {
            const { check } = await import('@tauri-apps/plugin-updater');
            const update = await check();
            if (update) await update.downloadAndInstall();
          }}
        >
          Install update
        </Button>
      </div>
    );
  }

  if (!outdated) return null;

  return (
    <div
      role="alert"
      data-testid="update-banner"
      className="flex shrink-0 items-center justify-between gap-3 border-b border-amber-300 bg-amber-100 px-4 py-2 text-sm text-amber-900 dark:border-amber-500/40 dark:bg-amber-500/15 dark:text-amber-100"
    >
      <div className="flex items-center gap-2">
        <RefreshCw className="h-4 w-4" aria-hidden="true" />
        <span>
          A new version of <strong>ex</strong> has been deployed. Reload to pick up the latest changes.
        </span>
      </div>
      <Button
        variant="outline"
        size="sm"
        onClick={() => {
          const sep = window.location.search ? '&' : '?';
          window.location.href = `${window.location.pathname}${window.location.search}${sep}v=${Date.now()}`;
        }}
        data-testid="update-banner-reload"
      >
        Reload now
      </Button>
    </div>
  );
}
