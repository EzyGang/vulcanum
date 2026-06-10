import type { ComponentChildren, JSX } from 'preact';

interface LoginViewProps {
  description: string;
  children: ComponentChildren;
}

export const LoginView = ({ description, children }: LoginViewProps): JSX.Element => (
  <div class='flex flex-col items-center justify-center min-h-screen bg-bg-page'>
    <div class='flex flex-col gap-6 w-full max-w-sm bg-bg-card p-8 border border-border-base'>
      <h1 class='text-2xl font-semibold text-text-primary tracking-wide uppercase'>Vulcanum</h1>

      <p class='text-text-secondary text-sm'>{description}</p>

      {children}
    </div>
  </div>
);
