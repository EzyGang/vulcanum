import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import { ProviderFormFields } from '../../providers/ui/ProviderFormFields.view';
import { Button } from '../../shared/ui/Button.view';
import { Label } from '../../shared/ui/Label.view';

interface ProjectFormProviderStepProps {
  providers: IntegrationProvider[];
  providerId: Signal<string>;
  showProviderForm: Signal<boolean>;
  newProviderName: Signal<string>;
  newProviderUrl: Signal<string>;
  newProviderKey: Signal<string>;
  providerFormError: Signal<string | null>;
  providerSubmitting: Signal<boolean>;
  isEdit: boolean;
  submitting: Signal<boolean>;
  actions: {
    onProviderChange: (id: string) => void;
    onShowProviderForm: () => void;
    onCancelProviderForm: () => void;
    onCreateProvider: (e: Event) => void;
  };
}

export const ProjectFormProviderStep = ({
  providers,
  providerId,
  showProviderForm,
  newProviderName,
  newProviderUrl,
  newProviderKey,
  providerFormError,
  providerSubmitting,
  isEdit,
  submitting,
  actions
}: ProjectFormProviderStepProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <Label for='field-provider'>Provider</Label>
    {providers.length > 0 && !showProviderForm.value ? (
      <div class='flex items-center gap-2'>
        <select
          id='field-provider'
          value={providerId.value}
          onChange={(e) => {
            actions.onProviderChange((e.target as HTMLSelectElement).value);
          }}
          disabled={isEdit || submitting.value}
          class='bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full'
        >
          <option value=''>Select a provider</option>
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        {!isEdit && (
          <Button variant='ghost' onClick={actions.onShowProviderForm}>
            + New
          </Button>
        )}
      </div>
    ) : (
      <ProviderFormFields
        name={newProviderName}
        url={newProviderUrl}
        apiKey={newProviderKey}
        error={providerFormError}
        submitting={providerSubmitting}
        mode='create'
        onSave={actions.onCreateProvider}
        onCancel={actions.onCancelProviderForm}
      />
    )}
  </div>
);
