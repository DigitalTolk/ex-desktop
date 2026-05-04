import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';

const platform = vi.hoisted(() => ({
  IS_TAURI: true,
  clearServerUrl: vi.fn(),
  getServerUrl: vi.fn(),
  saveServerUrlAndLoad: vi.fn(),
}));

vi.mock('@/platform', () => platform);
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: () => ({ label: 'setup' }),
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    platform.getServerUrl.mockResolvedValue('');
    platform.saveServerUrlAndLoad.mockResolvedValue('https://chat.example.com');
  });

  it('renders the wrapper setup flow', async () => {
    render(<App />);

    expect(screen.getByText('Connect to your server')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByLabelText('Server URL')).toBeEnabled();
    });
  });

  it('saves the server url and opens the remote workspace', async () => {
    render(<App />);

    const input = await screen.findByLabelText('Server URL');
    fireEvent.change(input, { target: { value: 'https://chat.example.com/' } });
    fireEvent.click(screen.getByRole('button', { name: 'Open workspace' }));

    await waitFor(() => {
      expect(platform.saveServerUrlAndLoad).toHaveBeenCalledWith('https://chat.example.com');
    });
    expect(screen.getByText('Opening your workspace…')).toBeInTheDocument();
  });

  it('requires a workspace url before submitting', async () => {
    render(<App />);

    await screen.findByLabelText('Server URL');
    fireEvent.click(screen.getByRole('button', { name: 'Open workspace' }));

    expect(
      await screen.findByText('Enter the full workspace URL, including https://'),
    ).toBeInTheDocument();
    expect(platform.saveServerUrlAndLoad).not.toHaveBeenCalled();
  });

  it('rejects unsupported workspace url schemes', async () => {
    render(<App />);

    const input = await screen.findByLabelText('Server URL');
    fireEvent.change(input, { target: { value: 'ftp://chat.example.com' } });
    fireEvent.click(screen.getByRole('button', { name: 'Open workspace' }));

    expect(await screen.findByText('Enter a valid http(s) workspace URL.')).toBeInTheDocument();
    expect(platform.saveServerUrlAndLoad).not.toHaveBeenCalled();
  });

  it('shows save errors from the native layer', async () => {
    platform.saveServerUrlAndLoad.mockRejectedValue(new Error('Could not save'));
    render(<App />);

    const input = await screen.findByLabelText('Server URL');
    fireEvent.change(input, { target: { value: 'https://chat.example.com' } });
    fireEvent.click(screen.getByRole('button', { name: 'Open workspace' }));

    expect(await screen.findByText('Could not save')).toBeInTheDocument();
  });

  it('clears the stored workspace', async () => {
    platform.getServerUrl.mockResolvedValue('https://chat.example.com');

    render(<App />);

    await screen.findByDisplayValue('https://chat.example.com');
    fireEvent.click(screen.getByRole('button', { name: 'Clear saved workspace' }));

    await waitFor(() => {
      expect(platform.clearServerUrl).toHaveBeenCalledTimes(1);
    });
  });

  it('shows clear errors from the native layer', async () => {
    platform.getServerUrl.mockResolvedValue('https://chat.example.com');
    platform.clearServerUrl.mockRejectedValue(new Error('Could not clear'));

    render(<App />);

    await screen.findByDisplayValue('https://chat.example.com');
    fireEvent.click(screen.getByRole('button', { name: 'Clear saved workspace' }));

    expect(await screen.findByText('Could not clear')).toBeInTheDocument();
  });
});
