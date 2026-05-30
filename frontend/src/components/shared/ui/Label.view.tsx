import { clsx } from 'clsx';
import type { JSX } from 'preact';

interface LabelProps {
  children: string;
  for?: string;
  class?: string;
}

export const Label = ({ children, for: forAttr, class: classProp }: LabelProps): JSX.Element => (
  <label for={forAttr} class={clsx('text-text-muted text-xs uppercase tracking-wider', classProp)}>
    {children}
  </label>
);
