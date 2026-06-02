import { signal } from '@preact/signals';
import { instanceLogin } from '../services/auth/auth.service';
import { fetchApi } from '../utils/api/client';

export const STORAGE_KEY = 'vulcanum-auth-token';

const loadToken = (): string | null => localStorage.getItem(STORAGE_KEY);

export const accessToken = signal<string | null>(loadToken());

export const login = async (password: string): Promise<void> => {
  const { token } = await instanceLogin(password);
  accessToken.value = token;
  localStorage.setItem(STORAGE_KEY, token);
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
  localStorage.removeItem(STORAGE_KEY);
};
