import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo, IntegrationProvider } from '../../../types/projects';
import { Button } from '../../shared/ui/Button.view';
import { ProjectFormColumns } from './ProjectFormColumns.view';
import { ProjectFormProjectLookup } from './ProjectFormProjectLookup.view';
import { ProjectFormProviderStep } from './ProjectFormProviderStep.view';
import { ProjectFormTextFields } from './ProjectFormTextFields.view';

interface ProjectFormViewProps {
  data: {
    isEdit: boolean;
    providers: IntegrationProvider[];
    providerId: Signal<string>;
    kaneoProjectId: Signal<string>;
    enabled: Signal<boolean>;
    pickupColumn: Signal<string>;
    progressColumn: Signal<string>;
    targetColumn: Signal<string>;
    promptTemplate: Signal<string>;
    repoUrl: Signal<string>;
    agentsMd: Signal<string>;
    opencodeConfig: Signal<string>;
    githubToken: Signal<string>;
    columns: Signal<ColumnInfo[]>;
    columnsLoading: Signal<boolean>;
    lookupProjectName: Signal<string>;
    lookupError: Signal<string | null>;
    lookedUp: Signal<boolean>;
    showProviderForm: Signal<boolean>;
    newProviderName: Signal<string>;
    newProviderUrl: Signal<string>;
    newProviderKey: Signal<string>;
    newProviderType: Signal<string>;
    providerFormError: Signal<string | null>;
    providerSubmitting: Signal<boolean>;
  };
  status: {
    submitting: Signal<boolean>;
    formError: Signal<string | null>;
    projectLoading: boolean;
  };
  actions: {
    onLookup: () => void;
    onSubmit: (e: Event) => void;
    onCancel: () => void;
    onCreateProvider: (e: Event) => void;
    onShowProviderForm: () => void;
    onCancelProviderForm: () => void;
    onProviderChange: (id: string) => void;
    onProjectIdChange: (id: string) => void;
    onEnabledChange: (checked: boolean) => void;
    onPromptTemplateChange: (value: string) => void;
    onRepoUrlChange: (value: string) => void;
    onAgentsMdChange: (value: string) => void;
    onOpencodeConfigChange: (value: string) => void;
    onGithubTokenChange: (value: string) => void;
    onPickupColumnChange: (value: string) => void;
    onProgressColumnChange: (value: string) => void;
    onTargetColumnChange: (value: string) => void;
  };
}

export const ProjectFormView = ({
  data: d,
  status: { submitting, formError, projectLoading },
  actions: a
}: ProjectFormViewProps): JSX.Element => {
  const canShowLookup = d.isEdit || !!d.providerId.value;
  const canShowFields = d.isEdit || d.lookedUp.value;

  return (
    <div class='flex flex-col gap-8'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
        {d.isEdit ? 'Edit Project' : 'Connect Project'}
      </h2>

      {projectLoading && <div class='text-text-muted text-sm'>Loading project...</div>}

      {!projectLoading && (
        <form onSubmit={a.onSubmit} class='flex flex-col gap-6 max-w-2xl'>
          <ProjectFormProviderStep
            providers={d.providers}
            providerId={d.providerId}
            showProviderForm={d.showProviderForm}
            newProviderName={d.newProviderName}
            newProviderUrl={d.newProviderUrl}
            newProviderKey={d.newProviderKey}
            newProviderType={d.newProviderType}
            providerFormError={d.providerFormError}
            providerSubmitting={d.providerSubmitting}
            isEdit={d.isEdit}
            submitting={submitting}
            actions={{
              onProviderChange: a.onProviderChange,
              onShowProviderForm: a.onShowProviderForm,
              onCancelProviderForm: a.onCancelProviderForm,
              onCreateProvider: a.onCreateProvider,
              onNewProviderNameChange: (v: string) => {
                d.newProviderName.value = v;
              },
              onNewProviderUrlChange: (v: string) => {
                d.newProviderUrl.value = v;
              },
              onNewProviderKeyChange: (v: string) => {
                d.newProviderKey.value = v;
              },
              onNewProviderTypeChange: (v: string) => {
                d.newProviderType.value = v;
              }
            }}
          />

          {canShowLookup && (
            <ProjectFormProjectLookup
              kaneoProjectId={d.kaneoProjectId}
              lookupProjectName={d.lookupProjectName}
              lookupError={d.lookupError}
              columnsLoading={d.columnsLoading}
              isEdit={d.isEdit}
              submitting={submitting}
              actions={{
                onLookup: a.onLookup,
                onProjectIdChange: a.onProjectIdChange
              }}
            />
          )}

          {canShowFields && (
            <>
              <ProjectFormColumns
                enabled={d.enabled}
                pickupColumn={d.pickupColumn.value}
                progressColumn={d.progressColumn.value}
                targetColumn={d.targetColumn.value}
                columns={d.columns.value}
                columnsLoading={d.columnsLoading.value}
                submitting={submitting.value}
                onEnabledChange={a.onEnabledChange}
                onPickupColumnChange={a.onPickupColumnChange}
                onProgressColumnChange={a.onProgressColumnChange}
                onTargetColumnChange={a.onTargetColumnChange}
              />

              <ProjectFormTextFields
                promptTemplate={d.promptTemplate}
                repoUrl={d.repoUrl}
                agentsMd={d.agentsMd}
                opencodeConfig={d.opencodeConfig}
                githubToken={d.githubToken}
                submitting={submitting}
                onPromptTemplateChange={a.onPromptTemplateChange}
                onRepoUrlChange={a.onRepoUrlChange}
                onAgentsMdChange={a.onAgentsMdChange}
                onOpencodeConfigChange={a.onOpencodeConfigChange}
                onGithubTokenChange={a.onGithubTokenChange}
              />

              {formError.value && <div class='text-error text-sm'>{formError.value}</div>}

              <div class='flex items-center gap-3'>
                <Button type='submit' variant='primary' disabled={submitting.value}>
                  {submitting.value ? 'Saving...' : d.isEdit ? 'Update Project' : 'Create Project'}
                </Button>
                <Button
                  type='button'
                  variant='secondary'
                  onClick={a.onCancel}
                  disabled={submitting.value}
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
