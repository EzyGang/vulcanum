import { Select as BaseSelect } from '@base-ui/react/select';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';

interface SelectProps {
  items: SelectOption[];
  value: string;
  onValueChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  invalid?: boolean;
  id?: string;
  class?: string;
}

const TRIGGER_CLASS =
  'inline-flex items-center justify-between gap-2 w-full bg-bg-input border text-text-primary px-4 py-3 text-sm cursor-pointer transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus disabled:cursor-not-allowed disabled:opacity-50';

const POSITIONER_CLASS = 'z-[100]';

const POPUP_CLASS =
  'min-w-44 max-h-72 overflow-y-auto bg-bg-active border border-border-base py-1 text-text-primary shadow-modal';

const ITEM_CLASS =
  'flex w-full items-center whitespace-nowrap bg-transparent px-4 py-2 text-sm cursor-pointer transition-colors text-text-secondary outline-none data-highlighted:bg-bg-hover data-highlighted:text-text-primary data-selected:bg-bg-card data-selected:text-text-primary';

export const Select = ({
  items,
  value,
  onValueChange,
  placeholder,
  disabled,
  invalid,
  id,
  class: classProp
}: SelectProps): JSX.Element => {
  const selectedLabel = items.find((o) => o.value === value)?.label;

  return (
    <BaseSelect.Root
      value={value}
      onValueChange={(v) => onValueChange(v ?? '')}
      disabled={disabled}
    >
      <BaseSelect.Trigger
        id={id}
        class={clsx(
          TRIGGER_CLASS,
          invalid ? 'border-error' : 'border-border-base focus:border-border-focus',
          classProp
        )}
      >
        <BaseSelect.Value placeholder={placeholder}>
          {selectedLabel ?? placeholder}
        </BaseSelect.Value>
        <span class='text-text-muted text-xs shrink-0' aria-hidden='true'>
          ▼
        </span>
      </BaseSelect.Trigger>
      <BaseSelect.Portal>
        <BaseSelect.Positioner
          align='start'
          alignItemWithTrigger={false}
          sideOffset={4}
          class={POSITIONER_CLASS}
        >
          <BaseSelect.Popup class={POPUP_CLASS}>
            {items.map((option) => (
              <BaseSelect.Item key={option.value} value={option.value} class={ITEM_CLASS}>
                <BaseSelect.ItemText>{option.label}</BaseSelect.ItemText>
              </BaseSelect.Item>
            ))}
          </BaseSelect.Popup>
        </BaseSelect.Positioner>
      </BaseSelect.Portal>
    </BaseSelect.Root>
  );
};
