import { invoke, isTauri } from '@tauri-apps/api/core';

export const IS_TAURI = isTauri();

function asError(error: unknown, fallback: string): Error {
  if (error instanceof Error) return error;
  if (typeof error === 'string') return new Error(error);
  return new Error(fallback);
}

export async function getServerUrl(): Promise<string> {
  if (!IS_TAURI) return '';
  try {
    return (await invoke<string | null>('get_server_url')) ?? '';
  } catch {
    return '';
  }
}

export async function setServerUrl(url: string): Promise<string> {
  if (!IS_TAURI) return url;
  try {
    return await invoke<string>('set_server_url', { url });
  } catch (error) {
    throw asError(error, 'Could not save the workspace URL.');
  }
}

export async function saveServerUrlAndLoad(url: string): Promise<string> {
  if (!IS_TAURI) return url;
  try {
    return await invoke<string>('save_server_url_and_load', { url });
  } catch (error) {
    throw asError(error, 'Could not open the workspace.');
  }
}

export async function clearServerUrl(): Promise<void> {
  if (!IS_TAURI) return;
  try {
    await invoke('clear_server_url');
  } catch (error) {
    throw asError(error, 'Could not clear the workspace URL.');
  }
}

export async function setBadgeCount(count: number): Promise<void> {
  if (!IS_TAURI) return;
  try {
    await invoke('set_badge_count', { count });
  } catch {
    // The wrapper app can function without badge updates.
  }
}

export async function getAutostart(): Promise<boolean> {
  return false;
}

export async function setAutostart(_enabled: boolean): Promise<void> {
  return;
}

export async function getRefreshToken(): Promise<string | null> {
  return null;
}

export async function setRefreshToken(_token: string): Promise<void> {
  return;
}

export async function deleteRefreshToken(): Promise<void> {
  return;
}
