import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useLocation } from 'wouter-preact';

import { login } from '../../../stores/auth.store';
import { ApiError } from '../../../utils/api/client';

export const useLogin = () => {
  const password = useSignal('');
  const error = useSignal<string | null>(null);
  const loading = useSignal(false);

  const [_, setLocation] = useLocation();

  const handlePasswordChange = useCallback((e: Event) => {
    const target = e.target as HTMLInputElement;
    password.value = target.value;
    error.value = null;
  }, []);

  const handleSubmit = useCallback(async (e: Event) => {
    e.preventDefault();

    if (!password.value) {
      error.value = 'Password is required';
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      await login(password.value);
      password.value = '';
      setLocation('/');
    } catch (err) {
      if (err instanceof ApiError) {
        error.value = err.status === 401 ? 'Invalid password' : err.message;
      } else {
        error.value = 'Connection failed. Is the server running?';
      }
    } finally {
      loading.value = false;
    }
  }, []);

  return { password, error, loading, handlePasswordChange, handleSubmit };
};
