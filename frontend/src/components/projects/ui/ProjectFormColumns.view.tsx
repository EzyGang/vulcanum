import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../types/projects';
import { Label } from '../../shared/ui/Label.view';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';

interface ProjectFormColumnsProps {
  enabled: { value: boolean };
  pickupColumn: string;
  progressColumn: string;
  targetColumn: string;
  columns: ColumnInfo[];
  columnsLoading: boolean;
  submitting: boolean;
  onEnabledChange: (checked: boolean) => void;
  onPickupColumnChange: (value: string) => void;
  onProgressColumnChange: (value: string) => void;
  onTargetColumnChange: (value: string) => void;
}

export const ProjectFormColumns = ({
  enabled,
  pickupColumn,
  progressColumn,
  targetColumn,
  columns,
  columnsLoading,
  submitting,
  onEnabledChange,
  onPickupColumnChange,
  onProgressColumnChange,
  onTargetColumnChange
}: ProjectFormColumnsProps): JSX.Element => (
  <>
    <label for='field-enabled' class='flex items-center gap-2 cursor-pointer'>
      <input
        id='field-enabled'
        type='checkbox'
        checked={enabled.value}
        onChange={(e) => onEnabledChange((e.target as HTMLInputElement).checked)}
        disabled={submitting}
      />
      <Label for='field-enabled'>Enabled</Label>
    </label>

    <ProjectFormColumnSelect
      id='field-pickup-column'
      label='Pickup Column'
      value={pickupColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting}
      placeholderText='Select pickup column'
      onChange={onPickupColumnChange}
    />
    <ProjectFormColumnSelect
      id='field-progress-column'
      label='Progress Column'
      value={progressColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting}
      placeholderText='Select progress column'
      onChange={onProgressColumnChange}
    />
    <ProjectFormColumnSelect
      id='field-target-column'
      label='Target Column'
      value={targetColumn}
      columns={columns}
      columnsLoading={columnsLoading}
      disabled={submitting}
      placeholderText='Select target column'
      onChange={onTargetColumnChange}
    />
  </>
);
