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
  newProviderType: Signal<string>;
  providerFormError: Signal<string | null>;
  providerSubmitting: Signal<boolean>;
  isEdit: boolean;
  submitting: Signal<boolean>;
  actions: {
    onProviderChange: (id: string) => void;
    onShowProviderForm: () => void;
    onCancelProviderForm: () => void;
    onCreateProvider: (e: Event) => void;
    onNewProviderNameChange: (value: string) => void;
    onNewProviderUrlChange: (value: string) => void;
    onNewProviderKeyChange: (value: string) => void;
    onNewProviderTypeChange: (value: string) => void;
  };
}

export const ProjectFormProviderStep = ({
  providers,
  providerId,
  showProviderForm,
  newProviderName,
  newProviderUrl,
  newProviderKey,
  newProviderType,
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
        name={newProviderName.value}
        url={newProviderUrl.value}
        apiKey={newProviderKey.value}
        providerType={newProviderType.value}
        error={providerFormError.value}
        submitting={providerSubmitting.value}
        mode='create'
        onSave={actions.onCreateProvider}
        onCancel={actions.onCancelProviderForm}
        onNameChange={actions.onNewProviderNameChange}
        onUrlChange={actions.onNewProviderUrlChange}
        onApiKeyChange={actions.onNewProviderKeyChange}
        onProviderTypeChange={actions.onNewProviderTypeChange}
      />
    )}
  </div>
);
