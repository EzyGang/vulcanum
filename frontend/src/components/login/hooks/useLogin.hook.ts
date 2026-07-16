import { type Signal, useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  exchangeAuthCode,
  getAuthMode,
  getGithubLoginUrl
} from '../../../services/auth/auth.service';
import { acceptTokenPair, login } from '../../../stores/auth.store';
import { ApiError } from '../../../utils/api/client';

export type LoginMode = 'loading' | 'single-user' | 'github' | 'unavailable';

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
    mode: LoginMode;
    description: string;
  };
}

export const useLogin = (): LoginViewProps => {
  const password = useSignal('');
  const error = useSignal<string | null>(null);
  const loading = useSignal(false);
  const authMode = useSignal<LoginMode>('loading');

  const [_, setLocation] = useLocation();

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');

    getAuthMode()
      .then((mode) => {
        authMode.value = mode.isSingleUser ? 'single-user' : 'github';
      })
      .catch(() => {
        error.value = 'Connection failed. Is the server running?';
        authMode.value = 'unavailable';
      });

    if (code) {
      loading.value = true;
      exchangeAuthCode(code)
        .then((tokenPair) => acceptTokenPair(tokenPair, true))
        .then(() => setLocation('/'))
        .catch(() => {
          error.value = 'GitHub login failed';
        })
        .finally(() => {
          loading.value = false;
        });
    }
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

  const description = getLoginDescription(authMode.value);
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
      mode: authMode.value,
      description
    }
  };
};

const getLoginDescription = (mode: LoginMode): string => {
  switch (mode) {
    case 'single-user':
      return 'Enter the instance password to continue.';
    case 'github':
      return 'Sign in with GitHub to create or access your team.';
    case 'loading':
    case 'unavailable':
      return 'Checking the configured login method.';
  }
};
