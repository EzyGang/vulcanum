import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  accessToken,
  clearAuthState,
  REFRESH_STORAGE_KEY,
  refreshToken,
  STORAGE_KEY
} from '../stores/auth.store';
import { fetchApi, refreshAccessToken } from '../utils/api/client';

const jsonResponse = (body: unknown, status = 200): Response =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' }
  });

const seedTokens = (): void => {
  accessToken.value = 'old-access';
  refreshToken.value = 'old-refresh';
  localStorage.setItem(STORAGE_KEY, 'old-access');
  localStorage.setItem(REFRESH_STORAGE_KEY, 'old-refresh');
};

describe('API token refresh', () => {
  beforeEach(() => {
    clearAuthState();
    vi.restoreAllMocks();
    seedTokens();
  });

  it('shares one rotation across concurrent unauthorized requests', async () => {
    let protectedCalls = 0;
    let refreshCalls = 0;
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.endsWith('/auth/refresh')) {
          refreshCalls += 1;
          return jsonResponse({
            access_token: 'new-access',
            refresh_token: 'new-refresh',
            refresh_expires_at: '2030-01-01T00:00:00Z'
          });
        }

        protectedCalls += 1;
        return protectedCalls <= 2
          ? jsonResponse({ error: 'Invalid token' }, 401)
          : jsonResponse({ value: 'ok' });
      })
    );

    const [first, second] = await Promise.all([
      fetchApi<{ value: string }>('/protected'),
      fetchApi<{ value: string }>('/protected')
    ]);

    expect(first).toEqual({ value: 'ok' });
    expect(second).toEqual({ value: 'ok' });
    expect(refreshCalls).toBe(1);
    expect(protectedCalls).toBe(4);
    expect(accessToken.value).toBe('new-access');
    expect(refreshToken.value).toBe('new-refresh');
  });

  it('clears credentials when refresh is unauthorized', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) =>
        String(input).endsWith('/auth/refresh')
          ? jsonResponse({ error: 'Invalid refresh token' }, 401)
          : jsonResponse({ error: 'Invalid token' }, 401)
      )
    );

    await expect(fetchApi('/protected')).rejects.toMatchObject({ status: 401 });

    expect(accessToken.value).toBeNull();
    expect(refreshToken.value).toBeNull();
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
    expect(localStorage.getItem(REFRESH_STORAGE_KEY)).toBeNull();
  });

  it('preserves credentials when refresh fails transiently', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => jsonResponse({ error: 'Unavailable' }, 503))
    );

    await expect(refreshAccessToken()).rejects.toMatchObject({ status: 503 });

    expect(accessToken.value).toBe('old-access');
    expect(refreshToken.value).toBe('old-refresh');
  });

  it('does not restore a token pair after local logout', async () => {
    let resolveRefresh: ((response: Response) => void) | undefined;
    vi.stubGlobal(
      'fetch',
      vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            resolveRefresh = resolve;
          })
      )
    );

    const pendingRefresh = refreshAccessToken();
    clearAuthState();
    resolveRefresh?.(
      jsonResponse({
        access_token: 'late-access',
        refresh_token: 'late-refresh',
        refresh_expires_at: '2030-01-01T00:00:00Z'
      })
    );

    await expect(pendingRefresh).resolves.toBe(false);
    expect(accessToken.value).toBeNull();
    expect(refreshToken.value).toBeNull();
  });
});
