import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ColumnInfo } from '../../../types/projects';

interface ProjectFormViewProps {
  data: {
    isEdit: boolean;
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
    columnsFetched: Signal<boolean>;
  };
  status: {
    submitting: Signal<boolean>;
    formError: Signal<string | null>;
    projectLoading: boolean;
  };
  actions: {
    onKaneoIdChange: (value: string) => void;
    onSubmit: (e: Event) => void;
    onCancel: () => void;
  };
}

const selectClasses =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const inputClasses =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const textareaClasses =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full font-mono';
const labelClasses = 'text-text-muted text-xs uppercase tracking-wider';

interface ColumnSelectProps {
  id: string;
  label: string;
  value: Signal<string>;
  columns: Signal<ColumnInfo[]>;
  columnsLoading: Signal<boolean>;
  disabled: boolean;
  placeholderText: string;
}

const ColumnSelect = ({
  id,
  label,
  value,
  columns,
  columnsLoading,
  disabled,
  placeholderText
}: ColumnSelectProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <label for={id} class={labelClasses}>
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
        class={selectClasses}
      >
        <option value=''>{placeholderText}</option>
        {columns.value.map((col) => (
          <option key={col.id} value={col.slug}>
            {col.name}
          </option>
        ))}
      </select>
    )}
  </div>
);

export const ProjectFormView = ({
  data: {
    isEdit,
    kaneoProjectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    columns,
    columnsLoading,
    columnsFetched
  },
  status: { submitting, formError, projectLoading },
  actions: { onKaneoIdChange, onSubmit, onCancel }
}: ProjectFormViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <div class='flex items-center justify-between'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
        {isEdit ? 'Edit Project' : 'Connect Project'}
      </h2>
    </div>

    {projectLoading && <div class='text-text-muted text-sm'>Loading project...</div>}

    {!projectLoading && (
      <form onSubmit={onSubmit} class='flex flex-col gap-6 max-w-2xl'>
        <div class='flex flex-col gap-2'>
          <label for='field-kaneo-project-id' class={labelClasses}>
            Kaneo Project ID
          </label>
          <input
            id='field-kaneo-project-id'
            type='text'
            value={kaneoProjectId.value}
            onInput={(e) => onKaneoIdChange((e.target as HTMLInputElement).value)}
            placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
            disabled={isEdit || submitting.value}
            class={inputClasses}
          />
        </div>

        <div class='flex flex-col gap-2'>
          <label
            for='field-enabled'
            class='flex items-center gap-2 text-text-muted text-xs uppercase tracking-wider cursor-pointer'
          >
            <input
              id='field-enabled'
              type='checkbox'
              checked={enabled.value}
              onChange={(e) => {
                enabled.value = (e.target as HTMLInputElement).checked;
              }}
              disabled={submitting.value}
            />
            Enabled
          </label>
        </div>

        {!columnsLoading.value && columnsFetched.value && columns.value.length === 0 && (
          <span class='text-text-muted text-xs'>Enter a Kaneo Project ID to load columns</span>
        )}

        <ColumnSelect
          id='field-pickup-column'
          label='Pickup Column'
          value={pickupColumn}
          columns={columns}
          columnsLoading={columnsLoading}
          disabled={submitting.value}
          placeholderText='Select pickup column'
        />

        <ColumnSelect
          id='field-progress-column'
          label='Progress Column'
          value={progressColumn}
          columns={columns}
          columnsLoading={columnsLoading}
          disabled={submitting.value}
          placeholderText='Select progress column'
        />

        <ColumnSelect
          id='field-target-column'
          label='Target Column'
          value={targetColumn}
          columns={columns}
          columnsLoading={columnsLoading}
          disabled={submitting.value}
          placeholderText='Select target column'
        />

        <div class='flex flex-col gap-2'>
          <div class='flex flex-col gap-1'>
            <label for='field-prompt-template' class={labelClasses}>
              Prompt Template
            </label>
            <span class='text-text-muted text-xs'>
              Template used to generate agent prompts. Supports {'{{task_title}}'},{' '}
              {'{{task_body}}'}, and {'{{repo_url}}'} variables.
            </span>
          </div>
          <textarea
            id='field-prompt-template'
            value={promptTemplate.value}
            onInput={(e) => {
              promptTemplate.value = (e.target as HTMLTextAreaElement).value;
            }}
            disabled={submitting.value}
            rows={4}
            class={textareaClasses}
          />
        </div>

        <div class='flex flex-col gap-2'>
          <div class='flex flex-col gap-1'>
            <label for='field-repo-url' class={labelClasses}>
              Repo URL
            </label>
            <span class='text-text-muted text-xs'>
              Git repository URL. For private repositories, embed a PAT directly:
              https://&lt;token&gt;@github.com/org/repo.
            </span>
          </div>
          <input
            id='field-repo-url'
            type='text'
            value={repoUrl.value}
            onInput={(e) => {
              repoUrl.value = (e.target as HTMLInputElement).value;
            }}
            placeholder='https://github.com/org/repo'
            disabled={submitting.value}
            class={inputClasses}
          />
        </div>

        <div class='flex flex-col gap-2'>
          <div class='flex flex-col gap-1'>
            <label for='field-agents-md' class={labelClasses}>
              Agents.md
            </label>
            <span class='text-text-muted text-xs'>
              Project-specific instructions written to AGENTS.md in the agent's work directory.
              Define conventions, repository access, and setup steps here.
            </span>
          </div>
          <textarea
            id='field-agents-md'
            value={agentsMd.value}
            onInput={(e) => {
              agentsMd.value = (e.target as HTMLTextAreaElement).value;
            }}
            disabled={submitting.value}
            rows={6}
            class={textareaClasses}
          />
        </div>

        {formError.value && <div class='text-error text-sm'>{formError.value}</div>}

        <div class='flex items-center gap-3'>
          <button
            type='submit'
            disabled={submitting.value}
            class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider
                   px-4 py-3 hover:opacity-90 transition-opacity disabled:opacity-50'
          >
            {submitting.value ? 'Saving...' : isEdit ? 'Update Project' : 'Create Project'}
          </button>
          <button
            type='button'
            onClick={onCancel}
            disabled={submitting.value}
            class='border border-border-base text-text-primary text-sm font-medium uppercase tracking-wider
                   px-4 py-3 hover:bg-bg-hover transition-colors disabled:opacity-50'
          >
            Cancel
          </button>
        </div>
      </form>
    )}
  </div>
);
