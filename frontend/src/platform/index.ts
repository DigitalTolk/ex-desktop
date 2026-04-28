import { isTauri } from '@tauri-apps/api/core';
import { invoke } from '@tauri-apps/api/core';

export const IS_TAURI = isTauri();

/**
 * Returns the configured server URL.
 * In browser/dev-proxy mode this is always '' (relative URLs work via Vite proxy).
 * In production Tauri this is whatever the user configured on first launch.
 */
export async function getServerUrl(): Promise<string> {
    if (!IS_TAURI) return '';
    try {
        return (await invoke<string | null>('get_server_url')) ?? '';
    } catch {
        return '';
    }
}

/** Persists the server URL to the OS app-data store. */
export async function setServerUrl(url: string): Promise<void> {
    if (!IS_TAURI) return;
    await invoke('set_server_url', { url });
}
