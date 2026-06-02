import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../types/projects';
import { Label } from '../../shared/ui/Label.view';

interface ProjectFormColumnSelectProps {
  id: string;
  label: string;
  value: string;
  columns: ColumnInfo[];
  columnsLoading: boolean;
  disabled: boolean;
  placeholderText: string;
  onChange: (value: string) => void;
}

export const ProjectFormColumnSelect = ({
  id,
  label,
  value,
  columns,
  columnsLoading,
  disabled,
  placeholderText,
  onChange
}: ProjectFormColumnSelectProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <Label for={id}>{label}</Label>
    {columnsLoading ? (
      <span class='text-text-muted text-sm'>Loading columns...</span>
    ) : (
      <select
        id={id}
        value={value}
        onChange={(e) => {
          onChange((e.target as HTMLSelectElement).value);
        }}
        disabled={disabled || !columns.length}
        class='bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full'
      >
        <option value=''>{placeholderText}</option>
        {columns.map((col) => (
          <option key={col.id} value={col.slug}>
            {col.name} ({col.slug})
          </option>
        ))}
      </select>
    )}
  </div>
);
