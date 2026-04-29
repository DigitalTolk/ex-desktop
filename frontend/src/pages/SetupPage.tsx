import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useServer } from '@/context/ServerContext';

export default function SetupPage() {
  const { setServer } = useServer();
  const [url, setUrl] = useState('https://');
  const [error, setError] = useState('');
  const [isChecking, setIsChecking] = useState(false);

  async function handleConnect(e: React.FormEvent) {
    e.preventDefault();
    setError('');

    const trimmed = url.trim().replace(/\/$/, '');
    if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) {
      setError('Enter a valid URL starting with https://');
      return;
    }

    setIsChecking(true);
    try {
      // no-cors avoids CORS preflight issues (healthz has no CORS headers);
      // a network error still throws, confirming the server is unreachable.
      await fetch(`${trimmed}/healthz`, {
        mode: 'no-cors',
        signal: AbortSignal.timeout(8000),
      });
      await setServer(trimmed);
    } catch {
      setError('Could not reach the server. Check the URL and your connection.');
    } finally {
      setIsChecking(false);
    }
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      <div className="w-full max-w-sm space-y-8">
        <div className="space-y-2 text-center">
          <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-primary">
            <span className="text-2xl font-bold text-primary-foreground">ex</span>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight">Connect to a workspace</h1>
          <p className="text-sm text-muted-foreground">
            Enter the URL of your self-hosted ex server.
          </p>
        </div>

        <form onSubmit={handleConnect} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="server-url">Workspace URL</Label>
            <Input
              id="server-url"
              type="url"
              placeholder="https://chat.yourcompany.com"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              autoFocus
              autoComplete="url"
              disabled={isChecking}
            />
          </div>

          {error && (
            <p className="text-sm text-destructive">{error}</p>
          )}

          <Button
            type="submit"
            className="w-full"
            disabled={isChecking || !url.trim()}
          >
            {isChecking ? 'Connecting…' : 'Connect'}
          </Button>
        </form>
      </div>
    </div>
  );
}
