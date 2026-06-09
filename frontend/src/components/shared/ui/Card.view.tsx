import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';

type CardPadding = 'sm' | 'md' | 'lg';

interface CardProps {
  children: ComponentChildren;
  padding?: CardPadding;
  class?: string;
}

const PADDING_MAP: Record<CardPadding, string> = {
  sm: 'p-5',
  md: 'p-8',
  lg: 'p-12'
};

export const Card = ({ children, padding = 'sm', class: classProp }: CardProps): JSX.Element => (
  <div
    class={clsx(
      'bg-bg-card border border-border-base transition-colors duration-fast',
      PADDING_MAP[padding],
      classProp
    )}
  >
    {children}
  </div>
);
