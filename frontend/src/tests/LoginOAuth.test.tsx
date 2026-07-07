import { render, waitFor } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/auth/auth.service', () => ({
  exchangeAuthCode: vi.fn(),
  getAuthMode: vi.fn(),
  getGithubLoginUrl: vi.fn(() => '/api/v1/auth/github/start'),
  getMe: vi.fn(),
  instanceLogin: vi.fn()
}));

const setLocation = vi.fn();
vi.mock('wouter-preact', () => ({
  useLocation: () => ['/login', setLocation]
}));

import { useLogin } from '../components/login/hooks/useLogin.hook';
import { exchangeAuthCode, getAuthMode, getMe } from '../services/auth/auth.service';
import {
  accessToken,
  clearAuthState,
  currentUser,
  REFRESH_STORAGE_KEY,
  refreshToken,
  STORAGE_KEY,
  selectedTeamId,
  teams
} from '../stores/auth.store';

const LoginHookHarness = () => {
  const login = useLogin();

  return (
    <div>
      <span data-testid='mode'>{login.view.mode}</span>
      <span data-testid='loading'>{String(login.status.loading.value)}</span>
      <span data-testid='error'>{login.status.error.value ?? ''}</span>
    </div>
  );
};

describe('useLogin OAuth callback', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearAuthState();
    currentUser.value = null;
    teams.value = [];
    selectedTeamId.value = null;
    window.history.pushState({}, '', '/login');
    vi.mocked(getAuthMode).mockResolvedValue({ isSingleUser: false });
    vi.mocked(getMe).mockResolvedValue({
      user: { id: 'user-1', email: 'user@example.com' },
      teams: [{ id: 'team-1', name: 'Team 1' }],
      identities: []
    });
  });

  it('exchanges the callback code and stores the returned token pair', async () => {
    window.history.pushState({}, '', '/login?code=callback-code');
    vi.mocked(exchangeAuthCode).mockResolvedValue({
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      refreshExpiresAt: '2030-01-01T00:00:00Z'
    });

    render(<LoginHookHarness />);

    await waitFor(() => expect(exchangeAuthCode).toHaveBeenCalledWith('callback-code'));
    await waitFor(() => expect(accessToken.value).toBe('access-token'));

    expect(refreshToken.value).toBe('refresh-token');
    expect(localStorage.getItem(STORAGE_KEY)).toBe('access-token');
    expect(localStorage.getItem(REFRESH_STORAGE_KEY)).toBe('refresh-token');
    expect(currentUser.value).toEqual({ id: 'user-1', email: 'user@example.com' });
    expect(teams.value).toEqual([{ id: 'team-1', name: 'Team 1' }]);
    expect(selectedTeamId.value).toBe('team-1');
    expect(setLocation).toHaveBeenCalledWith('/');
  });
});
