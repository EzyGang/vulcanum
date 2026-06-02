import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';

const PROVIDER_TYPE_OPTIONS: { value: string; label: string }[] = [
  { value: 'kaneo', label: 'Kaneo' }
];

interface ProviderFormFieldsProps {
  name: string;
  url: string;
  apiKey: string;
  providerType: string;
  error: string | null;
  submitting: boolean;
  mode: 'create' | 'edit';
  onSave: (e: Event) => void;
  onCancel: () => void;
  onNameChange: (value: string) => void;
  onUrlChange: (value: string) => void;
  onApiKeyChange: (value: string) => void;
  onProviderTypeChange: (value: string) => void;
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
  onCancel,
  onNameChange,
  onUrlChange,
  onApiKeyChange,
  onProviderTypeChange
}: ProviderFormFieldsProps): JSX.Element => {
  const saveLabel =
    mode === 'create'
      ? submitting
        ? 'Creating...'
        : 'Create Provider'
      : submitting
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
          value={providerType}
          onChange={(e) => {
            onProviderTypeChange((e.target as HTMLSelectElement).value);
          }}
          disabled={submitting}
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
        value={name}
        onInput={(e) => {
          onNameChange((e.target as HTMLInputElement).value);
        }}
        placeholder='Provider name'
        disabled={submitting}
      />
      <Input
        type='text'
        value={url}
        onInput={(e) => {
          onUrlChange((e.target as HTMLInputElement).value);
        }}
        placeholder='Instance URL (e.g. cloud.kaneo.app)'
        disabled={submitting}
      />
      <Input
        type='password'
        value={apiKey}
        onInput={(e) => {
          onApiKeyChange((e.target as HTMLInputElement).value);
        }}
        placeholder='API key'
        disabled={submitting}
      />
      {error && <div class='text-error text-sm'>{error}</div>}
      <div class='flex items-center gap-2'>
        <Button variant='primary' onClick={onSave} disabled={submitting}>
          {saveLabel}
        </Button>
        <Button variant='secondary' onClick={onCancel} disabled={submitting}>
          Cancel
        </Button>
      </div>
    </div>
  );
};
