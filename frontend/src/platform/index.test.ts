import { afterEach, describe, expect, it, vi } from 'vitest';

async function loadPlatform(options: {
  isTauri: boolean;
  invoke?: ReturnType<typeof vi.fn>;
}) {
  vi.resetModules();
  const invoke = options.invoke ?? vi.fn();
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke,
    isTauri: () => options.isTauri,
  }));

  return {
    invoke,
    platform: await import('./index'),
  };
}

describe('platform', () => {
  afterEach(() => {
    vi.doUnmock('@tauri-apps/api/core');
  });

  it('uses local no-op behavior outside Tauri', async () => {
    const { invoke, platform } = await loadPlatform({ isTauri: false });

    await expect(platform.getServerUrl()).resolves.toBe('');
    await expect(platform.setServerUrl('https://chat.example.com')).resolves.toBe(
      'https://chat.example.com',
    );
    await expect(platform.saveServerUrlAndLoad('https://chat.example.com')).resolves.toBe(
      'https://chat.example.com',
    );
    await expect(platform.clearServerUrl()).resolves.toBeUndefined();
    await expect(platform.setBadgeCount(3)).resolves.toBeUndefined();
    await expect(platform.getAutostart()).resolves.toBe(false);
    await expect(platform.setAutostart(true)).resolves.toBeUndefined();
    await expect(platform.getRefreshToken()).resolves.toBeNull();
    await expect(platform.setRefreshToken('token')).resolves.toBeUndefined();
    await expect(platform.deleteRefreshToken()).resolves.toBeUndefined();
    expect(platform.IS_TAURI).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('invokes Tauri commands when running inside Tauri', async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce('https://stored.example.com')
      .mockResolvedValueOnce('https://saved.example.com')
      .mockResolvedValueOnce('https://loaded.example.com')
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined);
    const { platform } = await loadPlatform({ isTauri: true, invoke });

    await expect(platform.getServerUrl()).resolves.toBe('https://stored.example.com');
    await expect(platform.setServerUrl('https://saved.example.com')).resolves.toBe(
      'https://saved.example.com',
    );
    await expect(platform.saveServerUrlAndLoad('https://loaded.example.com')).resolves.toBe(
      'https://loaded.example.com',
    );
    await expect(platform.clearServerUrl()).resolves.toBeUndefined();
    await expect(platform.setBadgeCount(7)).resolves.toBeUndefined();

    expect(invoke).toHaveBeenNthCalledWith(1, 'get_server_url');
    expect(invoke).toHaveBeenNthCalledWith(2, 'set_server_url', {
      url: 'https://saved.example.com',
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'save_server_url_and_load', {
      url: 'https://loaded.example.com',
    });
    expect(invoke).toHaveBeenNthCalledWith(4, 'clear_server_url');
    expect(invoke).toHaveBeenNthCalledWith(5, 'set_badge_count', { count: 7 });
  });

  it('falls back to an empty server url when reading fails', async () => {
    const invoke = vi.fn().mockRejectedValue(new Error('read failed'));
    const { platform } = await loadPlatform({ isTauri: true, invoke });

    await expect(platform.getServerUrl()).resolves.toBe('');
  });

  it('normalizes Tauri command errors', async () => {
    const saveFailure = vi.fn().mockRejectedValueOnce('save failed');
    const { platform: savePlatform } = await loadPlatform({
      isTauri: true,
      invoke: saveFailure,
    });
    await expect(savePlatform.setServerUrl('https://chat.example.com')).rejects.toThrow(
      'save failed',
    );

    const loadFailure = vi.fn().mockRejectedValueOnce({ reason: 'unknown' });
    const { platform: loadPlatformModule } = await loadPlatform({
      isTauri: true,
      invoke: loadFailure,
    });
    await expect(
      loadPlatformModule.saveServerUrlAndLoad('https://chat.example.com'),
    ).rejects.toThrow('Could not open the workspace.');

    const clearFailure = vi.fn().mockRejectedValueOnce({ reason: 'unknown' });
    const { platform: clearPlatform } = await loadPlatform({
      isTauri: true,
      invoke: clearFailure,
    });
    await expect(clearPlatform.clearServerUrl()).rejects.toThrow(
      'Could not clear the workspace URL.',
    );
  });

  it('ignores badge update errors', async () => {
    const invoke = vi.fn().mockRejectedValue(new Error('badge failed'));
    const { platform } = await loadPlatform({ isTauri: true, invoke });

    await expect(platform.setBadgeCount(4)).resolves.toBeUndefined();
  });
});
