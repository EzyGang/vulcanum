import type { JSX } from 'preact';
import { Label } from '../../shared/ui/Label.view';
import { Select } from '../../shared/ui/Select.view';
import type { TaskBoardColumnSettingsData, TaskBoardRoleSelectData } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface RoleSelectProps {
  data: TaskBoardRoleSelectData;
  disabled: boolean;
}

interface TaskBoardColumnSettingsProps {
  data: TaskBoardColumnSettingsData;
  disabled: boolean;
}

const RoleSelect = ({ data, disabled }: RoleSelectProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <Label for={data.id}>{data.label}</Label>
    <Select
      id={data.id}
      value={data.value}
      onValueChange={data.onValueChange}
      disabled={disabled}
      items={data.options}
    />
  </div>
);

export const TaskBoardColumnSettings = ({
  data,
  disabled
}: TaskBoardColumnSettingsProps): JSX.Element => (
  <TaskBoardSettingsSection
    title='Board columns'
    description='Map provider columns to the board lifecycle. These roles drive pickup, in-progress, done, and review automation.'
    hasOverrides={data.hasOverrides}
  >
    {data.hasOptions ? (
      <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
        {data.roleSelects.map((roleSelect) => (
          <RoleSelect key={roleSelect.id} data={roleSelect} disabled={disabled} />
        ))}
      </div>
    ) : (
      <p class='text-xs text-text-muted'>Provider columns are unavailable for this project.</p>
    )}
  </TaskBoardSettingsSection>
);
