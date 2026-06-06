import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../../types/projects';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';

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
      <Select
        id={id}
        value={value}
        onChange={(e) => {
          onChange((e.target as HTMLSelectElement).value);
        }}
        disabled={disabled || !columns.length}
      >
        <option value=''>{placeholderText}</option>
        {columns.map((col) => (
          <option key={col.id} value={col.slug}>
            {col.name} ({col.slug})
          </option>
        ))}
      </Select>
    )}
  </div>
);
