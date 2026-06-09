import type { JSX } from 'preact';
import { Select } from '../../../shared/ui/Select.view';
import { useProjectFormLookupContext } from '../../context/ProjectFormLookupContext';

export const ProjectFormLookup = (): JSX.Element => {
  const l = useProjectFormLookupContext();

  return (
    <div class='flex flex-col gap-3'>
      <div class='flex flex-col gap-1'>
        <span class='text-text-secondary text-sm'>Workspace</span>
        <Select
          id='field-workspace'
          items={l.workspaceOptions.value}
          value={l.workspaceId.value}
          onValueChange={l.onWorkspaceChange}
          placeholder={l.workspacesLoading.value ? 'Loading...' : 'Select a workspace'}
          disabled={l.workspaceSelectDisabled.value}
        />
      </div>

      {l.workspaceId.value && (
        <div class='flex flex-col gap-1'>
          <span class='text-text-secondary text-sm'>Project</span>
          <Select
            id='field-project'
            items={l.projectOptions.value}
            value={l.externalProjectId.value}
            onValueChange={l.onProjectSelectById}
            placeholder={l.projectsLoading.value ? 'Loading...' : 'Select a project'}
            disabled={l.projectSelectDisabled.value}
          />
        </div>
      )}

      {l.lookupError.value && <div class='text-error text-sm'>{l.lookupError.value}</div>}
      {l.lookupProjectName.value && l.lookedUp.value && (
        <div class='text-success text-sm'>Project: {l.lookupProjectName.value}</div>
      )}
    </div>
  );
};
