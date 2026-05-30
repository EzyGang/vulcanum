import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';

const PROVIDER_TYPE_OPTIONS: { value: string; label: string }[] = [
  { value: 'kaneo', label: 'Kaneo' }
];

interface ProviderFormFieldsProps {
  name: Signal<string>;
  url: Signal<string>;
  apiKey: Signal<string>;
  providerType: Signal<string>;
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
  providerType,
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
      <div class='flex flex-col gap-2'>
        <Label for='field-provider-type'>Type</Label>
        <select
          id='field-provider-type'
          value={providerType.value}
          onChange={(e) => {
            providerType.value = (e.target as HTMLSelectElement).value;
          }}
          disabled={submitting.value}
          class='bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full'
        >
          {PROVIDER_TYPE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
      <Input
        type='text'
        value={name.value}
        onInput={(e) => {
          name.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='Provider name'
        disabled={submitting.value}
      />
      <Input
        type='text'
        value={url.value}
        onInput={(e) => {
          url.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='Instance URL (e.g. cloud.kaneo.app)'
        disabled={submitting.value}
      />
      <Input
        type='password'
        value={apiKey.value}
        onInput={(e) => {
          apiKey.value = (e.target as HTMLInputElement).value;
        }}
        placeholder='API key'
        disabled={submitting.value}
      />
      {error.value && <div class='text-error text-sm'>{error.value}</div>}
      <div class='flex items-center gap-2'>
        <Button variant='primary' onClick={onSave} disabled={submitting.value}>
          {saveLabel}
        </Button>
        <Button variant='secondary' onClick={onCancel} disabled={submitting.value}>
          Cancel
        </Button>
      </div>
    </div>
  );
};
