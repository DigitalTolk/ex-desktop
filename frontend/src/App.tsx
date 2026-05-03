import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import {
  IS_TAURI,
  clearServerUrl,
  getServerUrl,
  saveServerUrlAndLoad,
} from '@/platform';

function normalizeServerUrl(value: string): string {
  return value.trim().replace(/\/+$/, '');
}

export default function App() {
  const [serverUrl, setServerUrl] = useState('');
  const [storedServerUrl, setStoredServerUrl] = useState('');
  const [isLoading, setIsLoading] = useState(IS_TAURI);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState('');

  useEffect(() => {
    let active = true;
    getServerUrl()
      .then(async (url) => {
        if (!active) return;
        const normalized = normalizeServerUrl(url);
        setServerUrl(url);
        setStoredServerUrl(url);

        if (!IS_TAURI || !normalized) {
          return;
        }

        const [{ getCurrentWebviewWindow }] = await Promise.all([
          import('@tauri-apps/api/webviewWindow'),
        ]);
        const currentWindow = getCurrentWebviewWindow();
        if (currentWindow.label === 'main') {
          window.location.replace(normalized);
        }
      })
      .catch(() => {
        if (!active) return;
        setError('Could not read the saved server address.');
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });

    return () => {
      active = false;
    };
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const submittedUrl = normalizeServerUrl(serverUrl);
    setError('');
    setStatus('');

    if (!submittedUrl) {
      setError('Enter the full workspace URL, including https://');
      return;
    }

    let safeUrl: string;
    try {
      const parsedUrl = new URL(submittedUrl);
      if (parsedUrl.protocol !== 'https:' && parsedUrl.protocol !== 'http:') {
        setError('Enter a valid http(s) workspace URL.');
        return;
      }
      safeUrl = normalizeServerUrl(parsedUrl.toString());
    } catch {
      setError('Enter a valid workspace URL, including https://');
      return;
    }

    setIsSubmitting(true);
    try {
      if (IS_TAURI) {
        const savedUrl = await saveServerUrlAndLoad(safeUrl);
        setStoredServerUrl(savedUrl);
        setStatus('Opening your workspace…');
      } else {
        window.location.assign(safeUrl);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Could not open that workspace.');
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleClear() {
    setError('');
    setStatus('');
    setIsSubmitting(true);
    try {
      await clearServerUrl();
      setServerUrl('');
      setStoredServerUrl('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Could not clear the saved workspace.');
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(245,158,11,0.16),_transparent_38%),linear-gradient(180deg,_#fcfaf5_0%,_#f4efe4_100%)] text-slate-900">
      <div className="mx-auto flex min-h-screen w-full max-w-2xl items-center px-6 py-10 sm:px-8">
        <section className="w-full rounded-[2rem] border border-white/70 bg-white/85 p-8 shadow-[0_30px_80px_rgba(52,36,13,0.12)] backdrop-blur sm:p-10">
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-amber-700">
            ex Desktop
          </p>
          <h1 className="mt-4 font-heading text-3xl font-semibold tracking-tight sm:text-4xl">
            Connect to your server
          </h1>
          <p className="mt-3 text-sm leading-6 text-slate-600 sm:text-base">
            Enter the full URL for your ex workspace.
          </p>

          <form className="mt-8 space-y-5" onSubmit={handleSubmit}>
            {storedServerUrl && (
              <p className="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                Saved server: <span className="break-all font-medium text-slate-900">{storedServerUrl}</span>
              </p>
            )}

              <label className="block">
                <span className="mb-2 block text-sm font-medium text-slate-700">
                  Server URL
                </span>
                <input
                  autoFocus
                  type="url"
                  inputMode="url"
                  placeholder="https://chat.yourcompany.com"
                  value={serverUrl}
                  onChange={(event) => setServerUrl(event.target.value)}
                  className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-base text-slate-900 shadow-sm transition outline-none placeholder:text-slate-400 focus:border-amber-500 focus:ring-4 focus:ring-amber-100"
                  aria-label="Workspace URL"
                  disabled={isLoading || isSubmitting}
                />
              </label>

              <div className="flex flex-col gap-3 sm:flex-row">
                <button
                  type="submit"
                  disabled={isLoading || isSubmitting}
                  className="inline-flex min-h-12 items-center justify-center rounded-2xl bg-slate-950 px-5 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
                >
                  {isSubmitting ? 'Opening workspace…' : 'Open workspace'}
                </button>
                <button
                  type="button"
                  onClick={handleClear}
                  disabled={isLoading || isSubmitting || !storedServerUrl}
                  className="inline-flex min-h-12 items-center justify-center rounded-2xl border border-slate-300 bg-white px-5 text-sm font-medium text-slate-700 transition hover:border-slate-400 hover:bg-slate-50 disabled:cursor-not-allowed disabled:border-slate-200 disabled:text-slate-400"
                >
                  Clear saved workspace
                </button>
              </div>
            </form>

            {error && (
              <p className="mt-4 rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
                {error}
              </p>
            )}
            {status && !error && (
              <p className="mt-4 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                {status}
              </p>
            )}
          <p className="mt-6 text-sm text-slate-500">
            You can change the server later from the tray menu.
          </p>
        </section>
      </div>
    </main>
  );
}
