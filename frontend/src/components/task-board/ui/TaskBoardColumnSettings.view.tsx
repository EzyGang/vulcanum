import type { JSX } from 'preact';
import { useCallback, useMemo } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';
import type { TaskBoardActions, TaskBoardColumnRole, TaskBoardColumnRoles } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface RoleSelectProps {
  id: string;
  label: string;
  value: string;
  columnRole: TaskBoardColumnRole;
  options: SelectOption[];
  disabled: boolean;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
}

interface TaskBoardColumnSettingsProps {
  columnRoles: TaskBoardColumnRoles;
  statusOptions: SelectOption[];
  disabled: boolean;
  actions: Pick<TaskBoardActions, 'onSetColumnRole'>;
}

const RoleSelect = ({
  id,
  label,
  value,
  columnRole,
  options,
  disabled,
  onSetColumnRole
}: RoleSelectProps): JSX.Element => {
  const selectColumn = useCallback(
    (columnSlug: string) => {
      onSetColumnRole(columnSlug === '' ? null : columnSlug, columnRole);
    },
    [onSetColumnRole, columnRole]
  );

  return (
    <div class='flex flex-col gap-2'>
      <Label for={id}>{label}</Label>
      <Select
        id={id}
        value={value}
        onValueChange={selectColumn}
        disabled={disabled}
        items={options}
      />
    </div>
  );
};

export const TaskBoardColumnSettings = ({
  columnRoles,
  statusOptions,
  disabled,
  actions
}: TaskBoardColumnSettingsProps): JSX.Element => {
  const reviewOptions = useMemo(
    () => [{ value: '', label: 'No review pickup override' }, ...statusOptions],
    [statusOptions]
  );

  return (
    <TaskBoardSettingsSection
      title='Board columns'
      description='Map provider columns to the board lifecycle. These roles drive pickup, in-progress, done, and review automation.'
      hasOverrides={columnRoles.reviewPickupColumn !== null}
    >
      {statusOptions.length > 0 ? (
        <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
          <RoleSelect
            id='board-settings-pickup-column'
            label='Pickup column'
            value={columnRoles.pickupColumn}
            columnRole='pickup'
            options={statusOptions}
            disabled={disabled}
            onSetColumnRole={actions.onSetColumnRole}
          />
          <RoleSelect
            id='board-settings-progress-column'
            label='In-progress column'
            value={columnRoles.progressColumn}
            columnRole='progress'
            options={statusOptions}
            disabled={disabled}
            onSetColumnRole={actions.onSetColumnRole}
          />
          <RoleSelect
            id='board-settings-done-column'
            label='Done column'
            value={columnRoles.targetColumn}
            columnRole='done'
            options={statusOptions}
            disabled={disabled}
            onSetColumnRole={actions.onSetColumnRole}
          />
          <RoleSelect
            id='board-settings-review-pickup-column'
            label='Review pickup column'
            value={columnRoles.reviewPickupColumn ?? ''}
            columnRole='review'
            options={reviewOptions}
            disabled={disabled}
            onSetColumnRole={actions.onSetColumnRole}
          />
        </div>
      ) : (
        <p class='text-xs text-text-muted'>Provider columns are unavailable for this project.</p>
      )}
    </TaskBoardSettingsSection>
  );
};
