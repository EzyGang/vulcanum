import type { JSX } from 'preact';
import type { LoginViewProps } from '../hooks/useLogin.hook';

export const LoginView = ({ view }: LoginViewProps): JSX.Element => (
  <div class='flex flex-col items-center justify-center min-h-screen bg-bg-page'>
    <div class='flex flex-col gap-6 w-full max-w-sm bg-bg-card p-8 border border-border-base'>
      <h1 class='text-2xl font-semibold text-text-primary tracking-wide uppercase'>Vulcanum</h1>

      <p class='text-text-secondary text-sm'>{view.description}</p>

      {view.content}
    </div>
  </div>
);
