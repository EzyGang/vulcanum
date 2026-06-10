import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';

interface LoginViewProps {
  data: {
    password: Signal<string>;
  };
  status: {
    error: Signal<string | null>;
    loading: Signal<boolean>;
    modeLoading: Signal<boolean>;
    isSingleUser: Signal<boolean>;
  };
  actions: {
    onPasswordChange: (e: Event) => void;
    onSubmit: (e: Event) => void;
    onGithubLogin: () => void;
  };
}

export const LoginView = ({
  data: { password },
  status: { error, loading, modeLoading, isSingleUser },
  actions: { onPasswordChange, onSubmit, onGithubLogin }
}: LoginViewProps): JSX.Element => (
  <div class='flex flex-col items-center justify-center min-h-screen bg-bg-page'>
    <div class='flex flex-col gap-6 w-full max-w-sm bg-bg-card p-8 border border-border-base'>
      <h1 class='text-2xl font-semibold text-text-primary tracking-wide uppercase'>Vulcanum</h1>

      <p class='text-text-secondary text-sm'>
        {isSingleUser.value
          ? 'Enter the instance password to continue.'
          : 'Sign in with GitHub to create or access your team.'}
      </p>

      {modeLoading.value ? (
        <div class='text-text-muted text-sm'>Loading auth mode...</div>
      ) : isSingleUser.value ? (
        <form onSubmit={onSubmit} class='flex flex-col gap-4'>
          <Input
            type='password'
            value={password.value}
            onInput={onPasswordChange}
            placeholder='Instance password'
            autofocus
            disabled={loading.value}
          />

          {error.value && <div class='text-error text-sm'>{error.value}</div>}

          <Button type='submit' variant='primary' disabled={loading.value}>
            {loading.value ? 'Signing in...' : 'Sign in'}
          </Button>
        </form>
      ) : (
        <div class='flex flex-col gap-4'>
          {error.value && <div class='text-error text-sm'>{error.value}</div>}
          <Button type='button' variant='primary' disabled={loading.value} onClick={onGithubLogin}>
            {loading.value ? 'Signing in...' : 'Sign in with GitHub'}
          </Button>
        </div>
      )}
    </div>
  </div>
);
