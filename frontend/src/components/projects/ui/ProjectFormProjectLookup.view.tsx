import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';

const labelStyles = 'text-text-muted text-xs uppercase tracking-wider';
const inputStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';

interface ProjectFormProjectLookupProps {
  kaneoProjectId: Signal<string>;
  lookupProjectName: Signal<string>;
  lookupError: Signal<string | null>;
  columnsLoading: Signal<boolean>;
  isEdit: boolean;
  submitting: Signal<boolean>;
  actions: {
    onLookup: () => void;
    onProjectIdChange: (id: string) => void;
  };
}

export const ProjectFormProjectLookup = ({
  kaneoProjectId,
  lookupProjectName,
  lookupError,
  columnsLoading,
  isEdit,
  submitting,
  actions
}: ProjectFormProjectLookupProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <label for='field-kaneo-project-id' class={labelStyles}>
      Kaneo Project ID
    </label>
    <div class='flex items-center gap-2'>
      <input
        id='field-kaneo-project-id'
        type='text'
        value={kaneoProjectId.value}
        onInput={(e) => {
          actions.onProjectIdChange((e.target as HTMLInputElement).value);
        }}
        placeholder='e.g. k5s7dwb5f89anmaui2d814h9'
        disabled={isEdit || submitting.value}
        class={`flex-1 ${inputStyles}`}
      />
      {!isEdit && (
        <button
          type='button'
          onClick={actions.onLookup}
          disabled={!kaneoProjectId.value || columnsLoading.value}
          class='border border-border-base text-text-primary text-sm uppercase tracking-wider px-4 py-3 hover:bg-bg-hover transition-colors disabled:opacity-50 whitespace-nowrap'
        >
          Look Up
        </button>
      )}
    </div>
    {lookupError.value && <div class='text-error text-sm'>{lookupError.value}</div>}
    {lookupProjectName.value && (
      <div class='text-success text-sm'>Project: {lookupProjectName.value}</div>
    )}
  </div>
);
