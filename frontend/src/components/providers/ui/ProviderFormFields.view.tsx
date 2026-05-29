import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';

const inputClasses =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';

interface ProviderFormFieldsProps {
  name: Signal<string>;
  url: Signal<string>;
  apiKey: Signal<string>;
  error: Signal<string | null>;
  submitting: Signal<boolean>;
  mode: 'create' | 'edit';
  onSave: (e: Event) => void;
  onCancel: () => void;
}

export const ProviderFormFields = ({
  name,
  url,
  apiKey,
  error,
  submitting,
  mode,
  onSave,
  onCancel
}: ProviderFormFieldsProps): JSX.Element => {
  const saveLabel =
    mode === 'create'
      ? submitting.value
        ? 'Creating...'
        : 'Create Provider'
      : submitting.value
        ? 'Updating...'
        : 'Update Provider';

  return (
    <div class='flex flex-col gap-3 border border-border-base p-4'>
      <span class='text-text-primary text-sm font-medium'>
        {mode === 'create' ? 'New Provider' : 'Edit Provider'}
      </span>
      <input
        type='text'
        value={name.value}
        onInput={(e) => {
          name.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='Provider name'
        disabled={submitting.value}
        class={inputClasses}
      />
      <input
        type='text'
        value={url.value}
        onInput={(e) => {
          url.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='Instance URL (e.g. cloud.kaneo.app)'
        disabled={submitting.value}
        class={inputClasses}
      />
      <input
        type='password'
        value={apiKey.value}
        onInput={(e) => {
          apiKey.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='API key'
        disabled={submitting.value}
        class={inputClasses}
      />
      {error.value && <div class='text-error text-sm'>{error.value}</div>}
      <div class='flex items-center gap-2'>
        <button
          type='button'
          onClick={onSave}
          disabled={submitting.value}
          class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider px-4 py-2 hover:opacity-90 transition-opacity disabled:opacity-50'
        >
          {saveLabel}
        </button>
        <button
          type='button'
          onClick={onCancel}
          disabled={submitting.value}
          class='border border-border-base text-text-primary text-sm uppercase tracking-wider px-4 py-2 hover:bg-bg-hover transition-colors disabled:opacity-50'
        >
          Cancel
        </button>
      </div>
    </div>
  );
};
