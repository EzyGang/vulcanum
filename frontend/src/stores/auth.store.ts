import { signal } from '@preact/signals';
import { getMe, instanceLogin } from '../services/auth/auth.service';
import type { AuthTeam, AuthUser } from '../types/auth';
import { fetchApi } from '../utils/api/client';

export const STORAGE_KEY = 'vulcanum-auth-token';
export const REFRESH_STORAGE_KEY = 'vulcanum-refresh-token';
export const TEAM_STORAGE_KEY = 'vulcanum-team-id';

const loadToken = (): string | null => localStorage.getItem(STORAGE_KEY);
const loadRefreshToken = (): string | null => localStorage.getItem(REFRESH_STORAGE_KEY);

export const accessToken = signal<string | null>(loadToken());
export const refreshToken = signal<string | null>(loadRefreshToken());
export const currentUser = signal<AuthUser | null>(null);
export const teams = signal<AuthTeam[]>([]);
export const selectedTeamId = signal<string | null>(localStorage.getItem(TEAM_STORAGE_KEY));

export const setSelectedTeamId = (teamId: string): void => {
  selectedTeamId.value = teamId;
  localStorage.setItem(TEAM_STORAGE_KEY, teamId);
};

const clearSessionState = (): void => {
  currentUser.value = null;
  teams.value = [];
  selectedTeamId.value = null;
  localStorage.removeItem(TEAM_STORAGE_KEY);
};

export const clearAuthState = (): void => {
  accessToken.value = null;
  refreshToken.value = null;
  clearSessionState();
  localStorage.removeItem(STORAGE_KEY);
  localStorage.removeItem(REFRESH_STORAGE_KEY);
};

export const acceptToken = async (
  token: string,
  loadUser = true,
  newRefreshToken?: string
): Promise<void> => {
  accessToken.value = token;
  localStorage.setItem(STORAGE_KEY, token);
  if (newRefreshToken) {
    refreshToken.value = newRefreshToken;
    localStorage.setItem(REFRESH_STORAGE_KEY, newRefreshToken);
  } else {
    refreshToken.value = null;
    clearSessionState();
    localStorage.removeItem(REFRESH_STORAGE_KEY);
  }
  if (loadUser) {
    await loadSession();
  }
};

export const loadSession = async (): Promise<void> => {
  if (!accessToken.value) return;

  const me = await getMe();
  currentUser.value = me.user;
  teams.value = me.teams;

  const currentTeamStillAvailable = me.teams.some((team) => team.id === selectedTeamId.value);
  if (!currentTeamStillAvailable && me.teams[0]) {
    setSelectedTeamId(me.teams[0].id);
  }
};

export const login = async (password: string): Promise<void> => {
  const { token } = await instanceLogin(password);
  await acceptToken(token, false);
};

export const logout = async (): Promise<void> => {
  const token = accessToken.value;
  const tokenToRevoke = refreshToken.value;
  if (token) {
    try {
      await fetchApi('/auth/logout', {
        method: 'POST',
        body: {
          refreshToken: tokenToRevoke
        }
      });
    } catch {
      // Local cleanup still removes the refresh token if the access token already expired.
    }
  }
  clearAuthState();
};
