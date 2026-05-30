import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../types/projects';
import { Label } from '../../shared/ui/Label.view';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';

interface ProjectFormColumnsProps {
  enabled: Signal<boolean>;
  pickupColumn: Signal<string>;
  progressColumn: Signal<string>;
  targetColumn: Signal<string>;
  columns: Signal<ColumnInfo[]>;
  columnsLoading: Signal<boolean>;
  submitting: Signal<boolean>;
  onEnabledChange: (checked: boolean) => void;
}

export const ProjectFormColumns = ({
  enabled,
  pickupColumn,
  progressColumn,
  targetColumn,
  columns,
  columnsLoading,
  submitting,
  onEnabledChange
}: ProjectFormColumnsProps): JSX.Element => (
  <>
    <label for='field-enabled' class='flex items-center gap-2 cursor-pointer'>
      <input
        id='field-enabled'
        type='checkbox'
        checked={enabled.value}
        onChange={(e) => onEnabledChange((e.target as HTMLInputElement).checked)}
        disabled={submitting.value}
      />
      <Label for='field-enabled'>Enabled</Label>
    </label>

    <ProjectFormColumnSelect
      id='field-pickup-column'
      label='Pickup Column'
      value={pickupColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting.value}
      placeholderText='Select pickup column'
    />
    <ProjectFormColumnSelect
      id='field-progress-column'
      label='Progress Column'
      value={progressColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting.value}
      placeholderText='Select progress column'
    />
    <ProjectFormColumnSelect
      id='field-target-column'
      label='Target Column'
      value={targetColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting.value}
      placeholderText='Select target column'
    />
  </>
);
