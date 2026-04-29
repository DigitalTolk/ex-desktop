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

/** Updates the tray icon badge with the total unread count. */
export async function setBadgeCount(count: number): Promise<void> {
    if (!IS_TAURI) return;
    try { await invoke('set_badge_count', { count }); } catch { /* ignore */ }
}

/** Returns whether launch-at-login is enabled. */
export async function getAutostart(): Promise<boolean> {
    if (!IS_TAURI) return false;
    try { return await invoke<boolean>('get_autostart'); } catch { return false; }
}

/** Enables or disables launch-at-login. */
export async function setAutostart(enabled: boolean): Promise<void> {
    if (!IS_TAURI) return;
    await invoke('set_autostart', { enabled });
}

/** Reads the refresh token from the OS keychain (Tauri only). */
export async function getRefreshToken(): Promise<string | null> {
    if (!IS_TAURI) return null;
    try { return await invoke<string | null>('get_refresh_token') ?? null; } catch { return null; }
}

/** Stores the refresh token in the OS keychain (Tauri only). */
export async function setRefreshToken(token: string): Promise<void> {
    if (!IS_TAURI) return;
    try { await invoke('set_refresh_token', { token }); } catch { /* ignore */ }
}

/** Removes the refresh token from the OS keychain (Tauri only). */
export async function deleteRefreshToken(): Promise<void> {
    if (!IS_TAURI) return;
    try { await invoke('delete_refresh_token'); } catch { /* ignore */ }
}
