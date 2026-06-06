import type { JSX } from 'preact';
import { useProjectFormContext } from '../../context/ProjectFormContext';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';

export const ProjectFormColumns = (): JSX.Element => {
  const { data: d, status, actions: a } = useProjectFormContext();

  return (
    <>
      <label for='field-enabled' class='flex items-center gap-2 cursor-pointer'>
        <input
          id='field-enabled'
          type='checkbox'
          checked={d.enabled.value}
          onChange={(e) => a.onEnabledChange((e.target as HTMLInputElement).checked)}
          disabled={status.submitting.value}
        />
        <span class='text-sm text-text-primary'>Enabled</span>
      </label>

      <ProjectFormColumnSelect
        id='field-pickup-column'
        label='Pickup Column'
        value={d.pickupColumn.value}
        columns={d.columns.value}
        columnsLoading={d.columnsLoading.value}
        disabled={status.submitting.value}
        placeholderText='Select pickup column'
        onChange={a.onPickupColumnChange}
      />
      <ProjectFormColumnSelect
        id='field-progress-column'
        label='Progress Column'
        value={d.progressColumn.value}
        columns={d.columns.value}
        columnsLoading={d.columnsLoading.value}
        disabled={status.submitting.value}
        placeholderText='Select progress column'
        onChange={a.onProgressColumnChange}
      />
      <ProjectFormColumnSelect
        id='field-target-column'
        label='Target Column'
        value={d.targetColumn.value}
        columns={d.columns.value}
        columnsLoading={d.columnsLoading.value}
        disabled={status.submitting.value}
        placeholderText='Select target column'
        onChange={a.onTargetColumnChange}
      />
    </>
  );
};
