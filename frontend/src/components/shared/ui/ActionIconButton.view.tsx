import type { ComponentChildren, JSX } from 'preact';
import { Button } from './Button.view';

interface ActionIconButtonProps {
  label: string;
  children: ComponentChildren;
  variant?: 'default' | 'danger' | 'critical' | 'success';
  disabled?: boolean;
  onClick?: JSX.MouseEventHandler<HTMLButtonElement>;
  type?: 'button' | 'submit';
}

const VARIANT_CLASS: Record<NonNullable<ActionIconButtonProps['variant']>, string> = {
  default: 'text-text-muted hover:text-text-primary',
  danger: 'text-text-muted hover:text-error',
  critical: 'text-error hover:text-error hover:opacity-90',
  success: 'text-success hover:text-success'
};

export const ActionIconButton = ({
  label,
  children,
  variant = 'default',
  disabled,
  onClick,
  type = 'button'
}: ActionIconButtonProps): JSX.Element => (
  <Button
    type={type}
    variant={variant === 'critical' ? 'danger' : 'ghost'}
    class={`h-10 w-10 border border-transparent hover:border-border-focus hover:bg-bg-active disabled:opacity-30 ${VARIANT_CLASS[variant]}`}
    title={label}
    aria-label={label}
    disabled={disabled}
    onClick={onClick}
  >
    {children}
  </Button>
);
