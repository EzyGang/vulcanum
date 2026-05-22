import { signal } from '@preact/signals';
import { instanceLogin } from '../services/auth/auth.service';

const STORAGE_KEY = 'vulcanum-auth-token';

const loadToken = (): string | null => localStorage.getItem(STORAGE_KEY);

export const accessToken = signal<string | null>(loadToken());

export const login = async (password: string): Promise<void> => {
  const { token } = await instanceLogin(password);
  accessToken.value = token;
  localStorage.setItem(STORAGE_KEY, token);
};

export const logout = (): void => {
  accessToken.value = null;
  localStorage.removeItem(STORAGE_KEY);
};
