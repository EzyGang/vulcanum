import { signal } from '@preact/signals';
import { getMe, instanceLogin } from '../services/auth/auth.service';
import type { AuthTeam, AuthUser } from '../types/auth';
import { fetchApi } from '../utils/api/client';

export const STORAGE_KEY = 'vulcanum-auth-token';
export const TEAM_STORAGE_KEY = 'vulcanum-team-id';

const loadToken = (): string | null => localStorage.getItem(STORAGE_KEY);

export const accessToken = signal<string | null>(loadToken());
export const currentUser = signal<AuthUser | null>(null);
export const teams = signal<AuthTeam[]>([]);
export const selectedTeamId = signal<string | null>(localStorage.getItem(TEAM_STORAGE_KEY));

export const setSelectedTeamId = (teamId: string): void => {
  selectedTeamId.value = teamId;
  localStorage.setItem(TEAM_STORAGE_KEY, teamId);
};

export const acceptToken = async (token: string, loadUser = true): Promise<void> => {
  accessToken.value = token;
  localStorage.setItem(STORAGE_KEY, token);
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
  if (token) {
    try {
      await fetchApi('/auth/logout', { method: 'POST' });
    } catch {
      // Token expires server-side in 15 minutes regardless
    }
  }
  accessToken.value = null;
  currentUser.value = null;
  teams.value = [];
  selectedTeamId.value = null;
  localStorage.removeItem(STORAGE_KEY);
  localStorage.removeItem(TEAM_STORAGE_KEY);
};
