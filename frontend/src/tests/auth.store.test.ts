import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/auth/auth.service', () => ({
  instanceLogin: vi.fn()
}));

import { instanceLogin } from '../services/auth/auth.service';
import { accessToken, login, logout } from '../stores/auth.store';

const TEST_KEY = 'vulcanum-auth-token';

describe('auth.store', () => {
  beforeEach(() => {
    localStorage.removeItem(TEST_KEY);
    accessToken.value = null;
    vi.clearAllMocks();
  });

  it('starts with no token when localStorage is empty', () => {
    expect(accessToken.value).toBeNull();
  });

  it('logout clears the token signal and localStorage', () => {
    accessToken.value = 'test-token';
    localStorage.setItem(TEST_KEY, 'test-token');

    logout();

    expect(accessToken.value).toBeNull();
    expect(localStorage.getItem(TEST_KEY)).toBeNull();
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
