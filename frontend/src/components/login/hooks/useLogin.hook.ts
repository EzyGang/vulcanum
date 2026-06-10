import { type Signal, useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';

import { getAuthMode, getGithubLoginUrl } from '../../../services/auth/auth.service';
import { acceptToken, login } from '../../../stores/auth.store';
import { ApiError } from '../../../utils/api/client';

type LoginMode = 'loading' | 'single-user' | 'github';

export interface LoginViewProps {
  data: {
    password: Signal<string>;
  };
  status: {
    error: Signal<string | null>;
    loading: Signal<boolean>;
  };
  actions: {
    onPasswordChange: (e: Event) => void;
    onSubmit: (e: Event) => void;
    onGithubLogin: () => void;
  };
  view: {
    description: string;
    mode: LoginMode;
  };
}

export const useLogin = (): LoginViewProps => {
  const password = useSignal('');
  const error = useSignal<string | null>(null);
  const loading = useSignal(false);
  const modeLoading = useSignal(true);
  const isSingleUser = useSignal(true);

  const [_, setLocation] = useLocation();

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const token = params.get('token');
    if (token) {
      loading.value = true;
      acceptToken(token)
        .then(() => setLocation('/'))
        .catch(() => {
          error.value = 'GitHub login failed';
        })
        .finally(() => {
          loading.value = false;
          modeLoading.value = false;
        });
      return;
    }

    getAuthMode()
      .then((mode) => {
        isSingleUser.value = mode.isSingleUser;
      })
      .catch(() => {
        error.value = 'Connection failed. Is the server running?';
      })
      .finally(() => {
        modeLoading.value = false;
      });
  }, []);

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

  const handleGithubLogin = useCallback(() => {
    window.location.href = getGithubLoginUrl();
  }, []);

  const mode: LoginMode = modeLoading.value
    ? 'loading'
    : isSingleUser.value
      ? 'single-user'
      : 'github';
  const description = isSingleUser.value
    ? 'Enter the instance password to continue.'
    : 'Sign in with GitHub to create or access your team.';

  return {
    data: {
      password
    },
    status: {
      error,
      loading
    },
    actions: {
      onPasswordChange: handlePasswordChange,
      onSubmit: handleSubmit,
      onGithubLogin: handleGithubLogin
    },
    view: {
      description,
      mode
    }
  };
};
