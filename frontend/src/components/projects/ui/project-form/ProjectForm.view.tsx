import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { useProjectFormContext } from '../../context/ProjectFormContext';
import { ProjectFormColumns } from './ProjectFormColumns.view';
import { ProjectFormLookup } from './ProjectFormLookup.view';
import { ProjectFormProviderStep } from './ProjectFormProviderStep.view';
import { ProjectFormTextFields } from './ProjectFormTextFields.view';

export const ProjectFormView = (): JSX.Element => {
  const { data: d, status, actions: a } = useProjectFormContext();

  return (
    <div class='flex flex-col gap-8'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
        {d.isEdit ? 'Edit Project' : 'Connect Project'}
      </h2>

      {status.projectLoading && <div class='text-text-muted text-sm'>Loading project...</div>}

      {!status.projectLoading && (
        <form onSubmit={a.onSubmit} class='flex flex-col gap-6 max-w-2xl'>
          <ProjectFormProviderStep />

          {d.canShowLookup && <ProjectFormLookup />}

          {d.canShowFields && (
            <>
              <ProjectFormColumns />
              <ProjectFormTextFields />

              {status.formError.value && (
                <div class='text-error text-sm'>{status.formError.value}</div>
              )}

              <div class='flex items-center gap-3'>
                <Button type='submit' variant='primary' disabled={status.submitting.value}>
                  {status.submitting.value
                    ? 'Saving...'
                    : d.isEdit
                      ? 'Update Project'
                      : 'Create Project'}
                </Button>
                <Button
                  type='button'
                  variant='secondary'
                  onClick={a.onCancel}
                  disabled={status.submitting.value}
                >
                  Cancel
                </Button>
              </div>
            </>
          )}
        </form>
      )}
    </div>
  );
};
