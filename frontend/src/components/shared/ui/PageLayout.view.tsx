import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';

type PageMaxWidth = '3xl' | '5xl' | '6xl' | '7xl' | 'board';

interface PageLayoutProps {
  children: ComponentChildren;
  maxWidth?: PageMaxWidth;
  gap?: 6 | 8;
  class?: string;
}

const MAX_WIDTH: Record<PageMaxWidth, string> = {
  '3xl': 'w-full max-w-3xl',
  '5xl': 'w-full max-w-5xl',
  '6xl': 'w-full max-w-6xl',
  '7xl': 'w-full max-w-7xl',
  board: 'w-full lg:w-[80vw] lg:max-w-[1536px]'
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
      'flex flex-col flex-1 px-4 sm:px-6 py-8 mx-auto animate-fade-in',
      MAX_WIDTH[maxWidth],
      GAP[gap],
      classProp
    )}
  >
    {children}
  </div>
);
