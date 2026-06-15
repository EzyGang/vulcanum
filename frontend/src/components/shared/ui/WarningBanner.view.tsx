import { clsx } from 'clsx';
import type { JSX } from 'preact';

interface WarningBannerProps {
  message: string;
  class?: string;
}

export const WarningBanner = ({ message, class: classProp }: WarningBannerProps): JSX.Element => (
  <div
    class={clsx('border border-warning-border bg-warning-bg p-4 text-sm text-warning', classProp)}
  >
    {message}
  </div>
);
