import { IconRefresh } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';

interface OverrideResetButtonProps {
  label: string;
  disabled: boolean;
  onClick: () => void;
}

export const OverrideResetButton = ({
  label,
  disabled,
  onClick
}: OverrideResetButtonProps): JSX.Element => (
  <Button
    type='button'
    variant='ghost'
    class='h-6 w-6 text-text-muted hover:text-text-primary disabled:opacity-30'
    title={label}
    aria-label={label}
    disabled={disabled}
    onClick={onClick}
  >
    <IconRefresh size={16} stroke={1.75} aria-hidden='true' />
  </Button>
);
