import { Checkbox as BaseCheckbox } from '@base-ui/react/checkbox';
import { clsx } from 'clsx';
import type { JSX } from 'preact';

interface CheckboxProps {
  id?: string;
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  indeterminate?: boolean;
  class?: string;
}

export const Checkbox = ({
  id,
  checked,
  onCheckedChange,
  disabled,
  indeterminate,
  class: classProp
}: CheckboxProps): JSX.Element => (
  <BaseCheckbox.Root
    id={id}
    checked={checked}
    onCheckedChange={onCheckedChange}
    disabled={disabled}
    class={clsx(
      'inline-flex items-center justify-center w-4 h-4 border rounded-none transition-colors duration-fast',
      'focus-visible:ring-2 focus-visible:ring-border-focus focus-visible:outline-none',
      disabled && 'opacity-50 cursor-not-allowed',
      checked || indeterminate
        ? 'bg-accent border-accent'
        : 'bg-bg-card border-border-base hover:border-border-focus',
      classProp
    )}
  >
    <BaseCheckbox.Indicator className='flex items-center justify-center w-full h-full'>
      {indeterminate ? (
        <span class='block w-2 h-0.5 bg-bg-page rounded-none' />
      ) : (
        <svg
          width='10'
          height='10'
          viewBox='0 0 10 10'
          fill='none'
          class='text-bg-page'
          aria-hidden='true'
        >
          <path
            d='M2 5L4 7L8 3'
            stroke='currentColor'
            strokeWidth='1.5'
            strokeLinecap='round'
            strokeLinejoin='round'
          />
        </svg>
      )}
    </BaseCheckbox.Indicator>
  </BaseCheckbox.Root>
);
