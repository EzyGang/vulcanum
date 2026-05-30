import { clsx } from 'clsx';
import type { JSX } from 'preact';

interface ErrorBannerProps {
  message: string;
  class?: string;
}

export const ErrorBanner = ({ message, class: classProp }: ErrorBannerProps): JSX.Element => (
  <div class={clsx('text-error text-sm bg-error-bg border border-error-border p-4', classProp)}>
    {message}
  </div>
);
