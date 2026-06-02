import { Button as BaseButton } from '@base-ui/react/button';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import type { ComponentPropsWithoutRef } from 'preact/compat';

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'ghost-danger';

interface ButtonProps extends Omit<ComponentPropsWithoutRef<'button'>, 'style' | 'className'> {
  variant?: ButtonVariant;
  class?: string;
}

const VARIANT_MAP: Record<ButtonVariant, string> = {
  primary:
    'bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider px-4 py-3 hover:opacity-90 transition-opacity disabled:opacity-50',
  secondary:
    'border border-border-base text-text-primary text-sm uppercase tracking-wider px-4 py-3 hover:bg-bg-hover transition-colors disabled:opacity-50',
  ghost:
    'text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors',
  danger: 'text-error text-xs uppercase tracking-wider hover:opacity-80 transition-opacity',
  'ghost-danger':
    'text-text-muted text-xs uppercase tracking-wider hover:text-error transition-colors'
};

export const Button = ({
  variant = 'secondary',
  class: classProp,
  ...rest
}: ButtonProps): JSX.Element => (
  <BaseButton
    className={clsx(
      'inline-flex items-center justify-center cursor-pointer transition-colors p-0 border-0 bg-transparent appearance-none',
      VARIANT_MAP[variant],
      classProp
    )}
    {...rest}
  />
);
