import { act, render } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../utils/api/client', () => ({
  refreshAccessToken: vi.fn(async () => true)
}));

import { useAuthRefresh } from '../hooks/useAuthRefresh.hook';
import { accessToken, clearAuthState, refreshToken } from '../stores/auth.store';
import { refreshAccessToken } from '../utils/api/client';

const HookHarness = () => {
  useAuthRefresh();
  return null;
};

const accessJwtExpiringAt = (expiresAt: number): string => {
  const payload = window
    .btoa(JSON.stringify({ exp: Math.floor(expiresAt / 1000) }))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  return `header.${payload}.signature`;
};

describe('useAuthRefresh', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
    vi.mocked(refreshAccessToken).mockClear();
    clearAuthState();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('refreshes one minute before access expiry', async () => {
    accessToken.value = accessJwtExpiringAt(Date.now() + 5 * 60_000);
    refreshToken.value = 'refresh-token';
    render(<HookHarness />);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4 * 60_000 - 1);
    });
    expect(refreshAccessToken).not.toHaveBeenCalled();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
  });

  it('refreshes immediately inside the lead-time window', async () => {
    accessToken.value = accessJwtExpiringAt(Date.now() + 30_000);
    refreshToken.value = 'refresh-token';
    render(<HookHarness />);

    await act(async () => {
      await vi.runOnlyPendingTimersAsync();
    });

    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
  });

  it('does not schedule without a usable token pair', async () => {
    accessToken.value = 'malformed-token';
    refreshToken.value = 'refresh-token';
    render(<HookHarness />);

    await act(async () => {
      await vi.runAllTimersAsync();
    });

    expect(refreshAccessToken).not.toHaveBeenCalled();
  });

  it('cancels the timer when unmounted', async () => {
    accessToken.value = accessJwtExpiringAt(Date.now() + 5 * 60_000);
    refreshToken.value = 'refresh-token';
    const view = render(<HookHarness />);

    view.unmount();
    await act(async () => {
      await vi.runAllTimersAsync();
    });

    expect(refreshAccessToken).not.toHaveBeenCalled();
  });
});
