import { clsx } from 'clsx';
import type { JSX } from 'preact';
import type { ComponentPropsWithoutRef } from 'preact/compat';

interface SelectProps extends Omit<ComponentPropsWithoutRef<'select'>, 'style' | 'className'> {
  invalid?: boolean;
  class?: string;
}

export const Select = ({ invalid, class: classProp, ...rest }: SelectProps): JSX.Element => (
  <select
    class={clsx(
      'w-full bg-bg-input border text-text-primary px-4 py-3 text-sm placeholder:text-text-muted focus:outline-none focus:border-border-focus transition-colors cursor-pointer',
      invalid ? 'border-error' : 'border-border-base',
      classProp
    )}
    {...rest}
  />
);
