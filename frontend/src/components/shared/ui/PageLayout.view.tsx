import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';

type PageMaxWidth = '3xl' | '5xl' | '6xl' | '7xl';

interface PageLayoutProps {
  children: ComponentChildren;
  maxWidth?: PageMaxWidth;
  gap?: 6 | 8;
  class?: string;
}

const MAX_WIDTH: Record<PageMaxWidth, string> = {
  '3xl': 'max-w-3xl',
  '5xl': 'max-w-5xl',
  '6xl': 'max-w-6xl',
  '7xl': 'max-w-7xl'
};

const GAP: Record<6 | 8, string> = {
  6: 'gap-6',
  8: 'gap-8'
};

export const PageLayout = ({
  children,
  maxWidth = '5xl',
  gap = 6,
  class: classProp
}: PageLayoutProps): JSX.Element => (
  <div
    class={clsx(
      'flex flex-col flex-1 px-4 sm:px-6 py-8 w-full mx-auto animate-fade-in',
      MAX_WIDTH[maxWidth],
      GAP[gap],
      classProp
    )}
  >
    {children}
  </div>
);
