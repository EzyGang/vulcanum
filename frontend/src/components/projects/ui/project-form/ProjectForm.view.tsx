import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { WarningBanner } from '../../../shared/ui/WarningBanner.view';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';
import { ProjectFormColumns } from './ProjectFormColumns.view';
import { ProjectFormLookup } from './ProjectFormLookup.view';
import { ProjectFormProviderStep } from './ProjectFormProviderStep.view';
import { ProjectFormTextFields } from './ProjectFormTextFields.view';

export const ProjectFormView = (): JSX.Element => {
  const m = useProjectFormMetaContext();

  return (
    <div class='flex flex-col gap-8'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
        {m.isEdit ? 'Edit Project' : 'Connect Project'}
      </h2>

      {m.projectSetupWarning && <WarningBanner message={m.projectSetupWarning} />}

      {m.projectLoading && <div class='text-text-muted text-sm'>Loading project...</div>}

      {!m.projectLoading && (
        <form onSubmit={m.onSubmit} class='flex flex-col gap-6 max-w-2xl'>
          <ProjectFormProviderStep />

          {m.canShowLookup && <ProjectFormLookup />}

          {m.canShowFields && (
            <>
              <ProjectFormColumns />
              <ProjectFormTextFields />

              {m.formError.value && <div class='text-error text-sm'>{m.formError.value}</div>}

              <div class='flex items-center gap-3'>
                <Button type='submit' variant='primary' disabled={m.submitting.value}>
                  {m.submitting.value
                    ? 'Saving...'
                    : m.isEdit
                      ? 'Update Project'
                      : 'Create Project'}
                </Button>
                <Button
                  type='button'
                  variant='secondary'
                  onClick={m.onCancel}
                  disabled={m.submitting.value}
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
