import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo, IntegrationProvider } from '../../../types/projects';
import { ProjectFormColumnSelect } from './ProjectFormColumnSelect.view';
import { ProjectFormProjectLookup } from './ProjectFormProjectLookup.view';
import { ProjectFormProviderStep } from './ProjectFormProviderStep.view';

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
    columns: Signal<ColumnInfo[]>;
    columnsLoading: Signal<boolean>;
    lookupProjectName: Signal<string>;
    lookupError: Signal<string | null>;
    lookedUp: Signal<boolean>;
    showProviderForm: Signal<boolean>;
    newProviderName: Signal<string>;
    newProviderUrl: Signal<string>;
    newProviderKey: Signal<string>;
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
  };
}

const labelStyles = 'text-text-muted text-xs uppercase tracking-wider';
const inputStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const textareaStyles = `${inputStyles} font-mono`;

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
            providerFormError={d.providerFormError}
            providerSubmitting={d.providerSubmitting}
            isEdit={d.isEdit}
            submitting={submitting}
            actions={{
              onProviderChange: a.onProviderChange,
              onShowProviderForm: a.onShowProviderForm,
              onCancelProviderForm: a.onCancelProviderForm,
              onCreateProvider: a.onCreateProvider
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
              <label for='field-enabled' class='flex items-center gap-2 cursor-pointer'>
                <input
                  id='field-enabled'
                  type='checkbox'
                  checked={d.enabled.value}
                  onChange={(e) => {
                    d.enabled.value = (e.target as HTMLInputElement).checked;
                  }}
                  disabled={submitting.value}
                />
                <span class={labelStyles}>Enabled</span>
              </label>

              <ProjectFormColumnSelect
                id='field-pickup-column'
                label='Pickup Column'
                value={d.pickupColumn}
                columns={d.columns}
                columnsLoading={d.columnsLoading}
                disabled={submitting.value}
                placeholderText='Select pickup column'
              />
              <ProjectFormColumnSelect
                id='field-progress-column'
                label='Progress Column'
                value={d.progressColumn}
                columns={d.columns}
                columnsLoading={d.columnsLoading}
                disabled={submitting.value}
                placeholderText='Select progress column'
              />
              <ProjectFormColumnSelect
                id='field-target-column'
                label='Target Column'
                value={d.targetColumn}
                columns={d.columns}
                columnsLoading={d.columnsLoading}
                disabled={submitting.value}
                placeholderText='Select target column'
              />

              <div class='flex flex-col gap-2'>
                <div class='flex flex-col gap-1'>
                  <label for='field-prompt-template' class={labelStyles}>
                    Prompt Template
                  </label>
                  <span class='text-text-muted text-xs'>
                    Supports {'{{task_title}}'}, {'{{task_body}}'}, and {'{{repo_url}}'} variables.
                  </span>
                </div>
                <textarea
                  id='field-prompt-template'
                  value={d.promptTemplate.value}
                  onInput={(e) => {
                    d.promptTemplate.value = (e.target as HTMLTextAreaElement).value;
                  }}
                  disabled={submitting.value}
                  rows={4}
                  class={textareaStyles}
                />
              </div>

              <div class='flex flex-col gap-2'>
                <label for='field-repo-url' class={labelStyles}>
                  Repo URL
                </label>
                <span class='text-text-muted text-xs'>
                  Git repository URL. Use https://&lt;token&gt;@github.com/org/repo for private
                  repos.
                </span>
                <input
                  id='field-repo-url'
                  type='text'
                  value={d.repoUrl.value}
                  onInput={(e) => {
                    d.repoUrl.value = (e.target as HTMLInputElement).value;
                  }}
                  placeholder='https://github.com/org/repo'
                  disabled={submitting.value}
                  class={inputStyles}
                />
              </div>

              <div class='flex flex-col gap-2'>
                <label for='field-agents-md' class={labelStyles}>
                  Agents.md
                </label>
                <span class='text-text-muted text-xs'>
                  Project instructions written to AGENTS.md in the work directory.
                </span>
                <textarea
                  id='field-agents-md'
                  value={d.agentsMd.value}
                  onInput={(e) => {
                    d.agentsMd.value = (e.target as HTMLTextAreaElement).value;
                  }}
                  disabled={submitting.value}
                  rows={6}
                  class={textareaStyles}
                />
              </div>

              {formError.value && <div class='text-error text-sm'>{formError.value}</div>}

              <div class='flex items-center gap-3'>
                <button
                  type='submit'
                  disabled={submitting.value}
                  class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider px-4 py-3 hover:opacity-90 transition-opacity disabled:opacity-50'
                >
                  {submitting.value ? 'Saving...' : d.isEdit ? 'Update Project' : 'Create Project'}
                </button>
                <button
                  type='button'
                  onClick={a.onCancel}
                  disabled={submitting.value}
                  class='border border-border-base text-text-primary text-sm font-medium uppercase tracking-wider px-4 py-3 hover:bg-bg-hover transition-colors disabled:opacity-50'
                >
                  Cancel
                </button>
              </div>
            </>
          )}
        </form>
      )}
    </div>
  );
};
