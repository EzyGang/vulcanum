import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo, IntegrationProvider } from '../../../types/projects';
import { ProviderCreateForm } from './ProviderCreateForm.view';

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
  };
}

const selectStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const inputStyles = selectStyles;
const textareaStyles = `${selectStyles} font-mono`;
const labelStyles = 'text-text-muted text-xs uppercase tracking-wider';

const ColumnSelect = ({
  id,
  label,
  value,
  columns,
  columnsLoading,
  disabled,
  placeholderText
}: {
  id: string;
  label: string;
  value: Signal<string>;
  columns: Signal<ColumnInfo[]>;
  columnsLoading: Signal<boolean>;
  disabled: boolean;
  placeholderText: string;
}): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <label for={id} class={labelStyles}>
      {label}
    </label>
    {columnsLoading.value ? (
      <span class='text-text-muted text-sm'>Loading columns...</span>
    ) : (
      <select
        id={id}
        value={value.value}
        onChange={(e) => {
          value.value = (e.target as HTMLSelectElement).value;
        }}
        disabled={disabled || !columns.value.length}
        class={selectStyles}
      >
        <option value=''>{placeholderText}</option>
        {columns.value.map((col) => (
          <option key={col.id} value={col.slug}>
            {col.name} ({col.slug})
          </option>
        ))}
      </select>
    )}
  </div>
);

export const ProjectFormView = ({
  data: d,
  status: { submitting, formError, projectLoading },
  actions: a
}: ProjectFormViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
      {d.isEdit ? 'Edit Project' : 'Connect Project'}
    </h2>

    {projectLoading && <div class='text-text-muted text-sm'>Loading project...</div>}

    {!projectLoading && (
      <form onSubmit={a.onSubmit} class='flex flex-col gap-6 max-w-2xl'>
        <div class='flex flex-col gap-2'>
          <label for='field-provider' class={labelStyles}>
            Provider
          </label>
          {d.providers.length > 0 && !d.showProviderForm.value ? (
            <div class='flex items-center gap-2'>
              <select
                id='field-provider'
                value={d.providerId.value}
                onChange={(e) => {
                  d.providerId.value = (e.target as HTMLSelectElement).value;
                }}
                disabled={d.isEdit || submitting.value}
                class={selectStyles}
              >
                <option value=''>Select a provider</option>
                {d.providers.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
              {!d.isEdit && (
                <button
                  type='button'
                  onClick={a.onShowProviderForm}
                  class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors whitespace-nowrap'
                >
                  + New
                </button>
              )}
            </div>
          ) : (
            <ProviderCreateForm
              name={d.newProviderName}
              url={d.newProviderUrl}
              apiKey={d.newProviderKey}
              error={d.providerFormError}
              submitting={d.providerSubmitting}
              onSave={a.onCreateProvider}
              onCancel={a.onCancelProviderForm}
            />
          )}
        </div>

        {(d.providerId.value || d.isEdit) && (
          <div class='flex flex-col gap-2'>
            <label for='field-kaneo-project-id' class={labelStyles}>
              Kaneo Project ID
            </label>
            <div class='flex items-center gap-2'>
              <input
                id='field-kaneo-project-id'
                type='text'
                value={d.kaneoProjectId.value}
                onInput={(e) => {
                  d.kaneoProjectId.value = (e.target as HTMLInputElement).value;
                }}
                placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
                disabled={d.isEdit || submitting.value}
                class={`flex-1 ${inputStyles}`}
              />
              {!d.isEdit && (
                <button
                  type='button'
                  onClick={a.onLookup}
                  disabled={!d.kaneoProjectId.value || d.columnsLoading.value}
                  class='border border-border-base text-text-primary text-sm uppercase tracking-wider px-4 py-3 hover:bg-bg-hover transition-colors disabled:opacity-50 whitespace-nowrap'
                >
                  Look Up
                </button>
              )}
            </div>
            {d.lookupError.value && <div class='text-error text-sm'>{d.lookupError.value}</div>}
            {d.lookupProjectName.value && (
              <div class='text-success text-sm'>Project: {d.lookupProjectName.value}</div>
            )}
          </div>
        )}

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

        <ColumnSelect
          id='field-pickup-column'
          label='Pickup Column'
          value={d.pickupColumn}
          columns={d.columns}
          columnsLoading={d.columnsLoading}
          disabled={submitting.value}
          placeholderText='Select pickup column'
        />
        <ColumnSelect
          id='field-progress-column'
          label='Progress Column'
          value={d.progressColumn}
          columns={d.columns}
          columnsLoading={d.columnsLoading}
          disabled={submitting.value}
          placeholderText='Select progress column'
        />
        <ColumnSelect
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
            Git repository URL. Use https://&lt;token&gt;@github.com/org/repo for private repos.
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
      </form>
    )}
  </div>
);
