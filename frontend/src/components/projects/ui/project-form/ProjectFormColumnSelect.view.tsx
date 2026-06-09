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
        onValueChange={onChange}
        disabled={disabled || !columns.length}
        placeholder={placeholderText}
        items={columns.map((col) => ({
          value: col.slug || col.name,
          label: col.slug ? `${col.name} (${col.slug})` : col.name
        }))}
      />
    )}
  </div>
);
