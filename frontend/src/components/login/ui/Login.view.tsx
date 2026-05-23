import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';

interface LoginViewProps {
  password: Signal<string>;
  error: Signal<string | null>;
  loading: Signal<boolean>;
  onPasswordChange: (e: Event) => void;
  onSubmit: (e: Event) => void;
}

export const LoginView = ({
  password,
  error,
  loading,
  onPasswordChange,
  onSubmit
}: LoginViewProps): JSX.Element => (
  <div class='flex flex-col items-center justify-center min-h-screen bg-bg-page'>
    <div class='flex flex-col gap-6 w-full max-w-sm bg-bg-card p-8 border border-border-base'>
      <h1 class='text-2xl font-semibold text-text-primary tracking-wide uppercase'>Vulcanum</h1>

      <p class='text-text-secondary text-sm'>Enter the instance password to continue.</p>

      <form onSubmit={onSubmit} class='flex flex-col gap-4'>
        <div class='flex flex-col gap-2'>
          <input
            type='password'
            value={password.value}
            onInput={onPasswordChange}
            placeholder='Instance password'
            autofocus
            disabled={loading.value}
            class='bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm placeholder:text-text-muted focus:outline-none focus:border-border-focus transition-colors'
          />
        </div>

        {error.value && <div class='text-error text-sm'>{error.value}</div>}

        <button
          type='submit'
          disabled={loading.value}
          class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider px-4 py-3 hover:opacity-90 transition-opacity disabled:opacity-50'
        >
          {loading.value ? 'Signing in...' : 'Sign in'}
        </button>
      </form>
    </div>
  </div>
);
