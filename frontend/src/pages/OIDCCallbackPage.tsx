import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '@/context/AuthContext';
import { setAccessToken, apiFetch, baseFetch } from '@/lib/api';
import { GENERAL_CHANNEL_SLUG } from '@/lib/roles';
import { useDocumentTitle } from '@/hooks/useDocumentTitle';
import { toast } from 'sonner';
import type { User } from '@/types';

export default function OIDCCallbackPage() {
  useDocumentTitle('Signing in…');
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { setAuth } = useAuth();

  useEffect(() => {
    async function handleCallback() {
      const hash = window.location.hash;
      let token: string | null = null;

      if (hash) {
        const hashParams = new URLSearchParams(hash.substring(1));
        token = hashParams.get('token') || hashParams.get('access_token');
      }

      if (!token) {
        token = searchParams.get('token') || searchParams.get('access_token');
      }

      toast.info(`[debug] OIDCCallback: token=${token ? token.slice(0, 20) + '…' : 'null'}`);

      if (!token) {
        try {
          const res = await baseFetch('/auth/token/refresh', {
            method: 'POST',
            credentials: 'include',
          });
          if (res.ok) {
            const data = await res.json();
            token = data.accessToken;
          }
        } catch {
          // ignore
        }
      }

      if (token) {
        setAccessToken(token);
        try {
          const user = await apiFetch<User>('/api/v1/users/me');
          setAuth(token, user);
          navigate(`/channel/${GENERAL_CHANNEL_SLUG}`, { replace: true });
          return;
        } catch (e) {
          toast.error(`[debug] /users/me failed: ${e}`);
        }
      }

      navigate('/login', { replace: true });
    }

    handleCallback();
  }, [navigate, searchParams, setAuth]);

  return (
    <div className="flex min-h-screen items-center justify-center">
      <p className="text-muted-foreground">Completing sign in...</p>
    </div>
  );
}
