import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../types/projects';

const selectStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const labelStyles = 'text-text-muted text-xs uppercase tracking-wider';

interface ProjectFormColumnSelectProps {
  id: string;
  label: string;
  value: Signal<string>;
  columns: Signal<ColumnInfo[]>;
  columnsLoading: Signal<boolean>;
  disabled: boolean;
  placeholderText: string;
}

export const ProjectFormColumnSelect = ({
  id,
  label,
  value,
  columns,
  columnsLoading,
  disabled,
  placeholderText
}: ProjectFormColumnSelectProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <label for={id} class={labelStyles}>
      {label}
    </label>
    {columnsLoading.value ? (
      <span class='text-text-muted text-sm'>Loading columns...</span>
    ) : (
      <select
        id={id}
        value={value.value}
        onChange={(e) => {
          value.value = (e.target as HTMLSelectElement).value;
        }}
        disabled={disabled || !columns.value.length}
        class={selectStyles}
      >
        <option value=''>{placeholderText}</option>
        {columns.value.map((col) => (
          <option key={col.id} value={col.slug}>
            {col.name} ({col.slug})
          </option>
        ))}
      </select>
    )}
  </div>
);
