import { Input as BaseInput } from '@base-ui/react/input';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import type { ComponentPropsWithoutRef } from 'preact/compat';

interface InputProps extends Omit<ComponentPropsWithoutRef<'input'>, 'style' | 'className'> {
  invalid?: boolean;
  class?: string;
}

export const Input = ({ invalid, class: classProp, ...rest }: InputProps): JSX.Element => (
  <BaseInput
    className={clsx(
      'w-full bg-bg-input border text-text-primary px-4 py-3 text-sm placeholder:text-text-muted focus:outline-none focus:border-border-focus transition-colors',
      invalid ? 'border-error' : 'border-border-base',
      classProp
    )}
    {...rest}
  />
);
