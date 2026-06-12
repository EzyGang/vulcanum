import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import type { LoginViewProps } from '../hooks/useLogin.hook';

export const LoginView = ({ data, status, actions, view }: LoginViewProps): JSX.Element => (
  <div class='flex flex-col items-center justify-center min-h-screen bg-bg-page'>
    <div class='flex flex-col gap-6 w-full max-w-sm bg-bg-card p-8 border border-border-base'>
      <h1 class='text-2xl font-semibold text-text-primary tracking-wide uppercase'>Vulcanum</h1>

      <p class='text-text-secondary text-sm'>{view.description}</p>

      {view.mode === 'loading' && <div class='text-text-muted text-sm'>Loading auth mode...</div>}

      {view.mode === 'unavailable' && (
        <div class='text-error text-sm'>{status.error.value || 'Unable to load auth mode.'}</div>
      )}

      {view.mode === 'single-user' && (
        <form onSubmit={actions.onSubmit} class='flex flex-col gap-4'>
          <Input
            type='password'
            value={data.password.value}
            onInput={actions.onPasswordChange}
            placeholder='Instance password'
            autofocus
            disabled={status.loading.value}
          />
          {status.error.value && <div class='text-error text-sm'>{status.error.value}</div>}
          <Button type='submit' variant='primary' disabled={status.loading.value}>
            {status.loading.value ? 'Signing in...' : 'Sign in'}
          </Button>
        </form>
      )}

      {view.mode === 'github' && (
        <div class='flex flex-col gap-4'>
          {status.error.value && <div class='text-error text-sm'>{status.error.value}</div>}
          <Button
            type='button'
            variant='primary'
            disabled={status.loading.value}
            onClick={actions.onGithubLogin}
          >
            {status.loading.value ? 'Signing in...' : 'Sign in with GitHub'}
          </Button>
        </div>
      )}
    </div>
  </div>
);
