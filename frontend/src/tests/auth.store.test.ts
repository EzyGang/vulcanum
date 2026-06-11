import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/auth/auth.service', () => ({
  instanceLogin: vi.fn(),
  getMe: vi.fn()
}));

vi.mock('../utils/api/client', () => ({
  fetchApi: vi.fn().mockResolvedValue(undefined)
}));

import { instanceLogin } from '../services/auth/auth.service';
import {
  acceptToken,
  accessToken,
  currentUser,
  login,
  logout,
  REFRESH_STORAGE_KEY,
  refreshToken,
  selectedTeamId,
  TEAM_STORAGE_KEY,
  teams
} from '../stores/auth.store';
import { fetchApi } from '../utils/api/client';

const TEST_KEY = 'vulcanum-auth-token';

describe('auth.store', () => {
  beforeEach(() => {
    localStorage.removeItem(TEST_KEY);
    localStorage.removeItem(REFRESH_STORAGE_KEY);
    localStorage.removeItem(TEAM_STORAGE_KEY);
    accessToken.value = null;
    refreshToken.value = null;
    currentUser.value = null;
    teams.value = [];
    selectedTeamId.value = null;
    vi.clearAllMocks();
  });

  it('starts with no token when localStorage is empty', () => {
    expect(accessToken.value).toBeNull();
  });

  it('logout clears the token signal and localStorage', async () => {
    accessToken.value = 'test-token';
    refreshToken.value = 'test-refresh-token';
    currentUser.value = { id: 'user-1', email: 'user@example.com' };
    teams.value = [{ id: 'team-1', name: 'Team 1' }];
    selectedTeamId.value = 'team-1';
    localStorage.setItem(TEST_KEY, 'test-token');
    localStorage.setItem(REFRESH_STORAGE_KEY, 'test-refresh-token');
    localStorage.setItem(TEAM_STORAGE_KEY, 'team-1');

    await logout();

    expect(accessToken.value).toBeNull();
    expect(refreshToken.value).toBeNull();
    expect(currentUser.value).toBeNull();
    expect(teams.value).toEqual([]);
    expect(selectedTeamId.value).toBeNull();
    expect(localStorage.getItem(TEST_KEY)).toBeNull();
    expect(localStorage.getItem(REFRESH_STORAGE_KEY)).toBeNull();
    expect(localStorage.getItem(TEAM_STORAGE_KEY)).toBeNull();
  });

  it('acceptToken without refresh support clears stale session and team state', async () => {
    refreshToken.value = 'old-refresh-token';
    currentUser.value = { id: 'old-user', email: 'old@example.com' };
    teams.value = [{ id: 'old-team', name: 'Old Team' }];
    selectedTeamId.value = 'old-team';
    localStorage.setItem(REFRESH_STORAGE_KEY, 'old-refresh-token');
    localStorage.setItem(TEAM_STORAGE_KEY, 'old-team');

    await acceptToken('instance-token', false);

    expect(accessToken.value).toBe('instance-token');
    expect(refreshToken.value).toBeNull();
    expect(currentUser.value).toBeNull();
    expect(teams.value).toEqual([]);
    expect(selectedTeamId.value).toBeNull();
    expect(localStorage.getItem(TEST_KEY)).toBe('instance-token');
    expect(localStorage.getItem(REFRESH_STORAGE_KEY)).toBeNull();
    expect(localStorage.getItem(TEAM_STORAGE_KEY)).toBeNull();
  });

  it('logout reads the current refresh token when the request is sent', async () => {
    accessToken.value = 'test-token';
    refreshToken.value = 'old-refresh-token';
    vi.mocked(fetchApi).mockImplementationOnce(async (_path, options) => {
      if (!options) return undefined;

      refreshToken.value = 'new-refresh-token';
      const body = typeof options.body === 'function' ? options.body() : options.body;
      expect(body).toEqual({ refreshToken: 'new-refresh-token' });
      return undefined;
    });

    await logout();

    expect(fetchApi).toHaveBeenCalledWith('/auth/logout', {
      method: 'POST',
      body: expect.any(Function)
    });
  });

  it('login sets token in signal and localStorage on success', async () => {
    const mockToken = 'mock-session-token-abc123';
    vi.mocked(instanceLogin).mockResolvedValue({ token: mockToken });

    await login('correct-password');

    expect(instanceLogin).toHaveBeenCalledWith('correct-password');
    expect(accessToken.value).toBe(mockToken);
    expect(localStorage.getItem(TEST_KEY)).toBe(mockToken);
  });

  it('login throws when password is wrong', async () => {
    vi.mocked(instanceLogin).mockRejectedValue(new Error('Invalid password'));

    await expect(login('wrong')).rejects.toThrow('Invalid password');
    expect(accessToken.value).toBeNull();
    expect(localStorage.getItem(TEST_KEY)).toBeNull();
  });
});
