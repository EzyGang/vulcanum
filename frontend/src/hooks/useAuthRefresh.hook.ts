import { useEffect } from 'preact/hooks';

import { accessToken, refreshToken } from '../stores/auth.store';
import { refreshAccessToken } from '../utils/api/client';

const REFRESH_LEAD_TIME_MS = 60_000;

export const useAuthRefresh = (): void => {
  const currentAccessToken = accessToken.value;
  const currentRefreshToken = refreshToken.value;

  useEffect(() => {
    if (!currentAccessToken || !currentRefreshToken) return;

    const expiresAt = readJwtExpiry(currentAccessToken);
    if (expiresAt === null) return;

    const delay = Math.max(0, expiresAt - Date.now() - REFRESH_LEAD_TIME_MS);
    const timeout = window.setTimeout(() => {
      void refreshAccessToken().catch(() => undefined);
    }, delay);

    return () => window.clearTimeout(timeout);
  }, [currentAccessToken, currentRefreshToken]);
};

const readJwtExpiry = (token: string): number | null => {
  const payload = token.split('.')[1];
  if (!payload) return null;

  try {
    const normalized = payload.replace(/-/g, '+').replace(/_/g, '/');
    const padding = '='.repeat((4 - (normalized.length % 4)) % 4);
    const claims = JSON.parse(window.atob(`${normalized}${padding}`)) as { exp?: unknown };
    return typeof claims.exp === 'number' ? claims.exp * 1000 : null;
  } catch {
    return null;
  }
};
