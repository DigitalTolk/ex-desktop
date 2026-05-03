let accessToken: string | null = null;

// Seeded from localStorage on module load so getBaseUrl() is never empty
// even before ServerContext's async getServerUrl() resolves. ServerContext
// keeps this in sync via setBaseUrl whenever the configured URL changes.
// Skipped in Vite dev mode so the dev proxy continues to work.
const LS_KEY = 'ex_server_url';
let baseUrl = import.meta.env.DEV ? '' : (localStorage.getItem(LS_KEY) ?? '');

export function setBaseUrl(url: string) {
  baseUrl = url.replace(/\/$/, '');
  try { localStorage.setItem(LS_KEY, baseUrl); } catch { /* ignore */ }
}

export function getBaseUrl(): string {
  return baseUrl;
}

/** Resolves a server-relative URL (e.g. /api/v1/media/…) against the
 *  configured base URL so <img> tags work in Tauri where the page origin
 *  is tauri://localhost rather than the API server. Absolute URLs pass through unchanged. */
export function resolveMediaUrl(url: string | undefined | null): string {
  if (!url) return '';
  if (url.startsWith('/')) return `${baseUrl}${url}`;
  return url;
}

export function setAccessToken(token: string) {
  accessToken = token;
}

export function getAccessToken(): string | null {
  return accessToken;
}

export function clearAccessToken() {
  accessToken = null;
}

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

/** fetch() wrapper that automatically prepends the configured server base URL. */
export function baseFetch(path: string, options?: RequestInit): Promise<Response> {
  return fetch(`${baseUrl}${path}`, options);
}

async function tryRefreshToken(): Promise<boolean> {
  try {
    const res = await baseFetch('/auth/token/refresh', {
      method: 'POST',
      credentials: 'include',
    });
    if (!res.ok) return false;
    const data = await res.json();
    if (data.accessToken) {
      setAccessToken(data.accessToken);
      return true;
    }
    return false;
  } catch {
    return false;
  }
}

export async function apiFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const headers = new Headers(options.headers);

  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`);
  }

  if (
    options.body &&
    typeof options.body === 'string' &&
    !headers.has('Content-Type')
  ) {
    headers.set('Content-Type', 'application/json');
  }

  const url = `${baseUrl}${path}`;

  const res = await fetch(url, {
    ...options,
    headers,
    credentials: 'include',
  });

  if (res.status === 401 && accessToken) {
    const refreshed = await tryRefreshToken();
    if (refreshed) {
      headers.set('Authorization', `Bearer ${accessToken}`);
      const retry = await fetch(url, {
        ...options,
        headers,
        credentials: 'include',
      });
      if (!retry.ok) {
        throw new ApiError(retry.status, await retry.text());
      }
      return retry.json();
    }
    clearAccessToken();
    throw new ApiError(401, 'Unauthorized');
  }

  if (!res.ok) {
    throw new ApiError(res.status, await res.text());
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}
