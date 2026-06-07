import type { JSX } from 'preact';
import { useProjectFormFieldsContext } from '../../context/ProjectFormFieldsContext';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';

export const ProjectFormColumns = (): JSX.Element => {
  const m = useProjectFormMetaContext();
  const f = useProjectFormFieldsContext();

  return (
    <>
      <label for='field-enabled' class='flex items-center gap-2 cursor-pointer'>
        <input
          id='field-enabled'
          type='checkbox'
          checked={f.enabled.value}
          onChange={(e) => f.onEnabledChange((e.target as HTMLInputElement).checked)}
          disabled={m.submitting.value}
        />
        <span class='text-sm text-text-primary'>Enabled</span>
      </label>

      <ProjectFormColumnSelect
        id='field-pickup-column'
        label='Pickup Column'
        value={f.pickupColumn.value}
        columns={f.columns.value}
        columnsLoading={f.columnsLoading.value}
        disabled={m.submitting.value}
        placeholderText='Select pickup column'
        onChange={f.onPickupColumnChange}
      />
      <ProjectFormColumnSelect
        id='field-progress-column'
        label='Progress Column'
        value={f.progressColumn.value}
        columns={f.columns.value}
        columnsLoading={f.columnsLoading.value}
        disabled={m.submitting.value}
        placeholderText='Select progress column'
        onChange={f.onProgressColumnChange}
      />
      <ProjectFormColumnSelect
        id='field-target-column'
        label='Target Column'
        value={f.targetColumn.value}
        columns={f.columns.value}
        columnsLoading={f.columnsLoading.value}
        disabled={m.submitting.value}
        placeholderText='Select target column'
        onChange={f.onTargetColumnChange}
      />
    </>
  );
};
