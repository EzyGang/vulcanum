import { clsx } from 'clsx';
import type { JSX } from 'preact';
import type { ComponentPropsWithoutRef } from 'preact/compat';

interface TextAreaProps extends Omit<ComponentPropsWithoutRef<'textarea'>, 'style' | 'className'> {
  invalid?: boolean;
  class?: string;
}

export const TextArea = ({ invalid, class: classProp, ...rest }: TextAreaProps): JSX.Element => (
  <textarea
    class={clsx(
      'w-full bg-bg-input border text-text-primary px-4 py-3 text-sm placeholder:text-text-muted focus:outline-none focus:border-border-focus transition-colors',
      invalid ? 'border-error' : 'border-border-base',
      classProp
    )}
    {...rest}
  />
);
