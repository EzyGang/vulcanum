import type { JSX } from 'preact';
import { ProviderFormFields } from '../../../providers/ui/ProviderFormFields.view';
import { Button } from '../../../shared/ui/Button.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { useProjectFormContext } from '../../context/ProjectFormContext';

export const ProjectFormProviderStep = (): JSX.Element => {
  const { data: d, status, actions: a } = useProjectFormContext();

  return (
    <div class='flex flex-col gap-2'>
      <Label for='field-provider'>Provider</Label>
      {d.providers.length > 0 && !d.showProviderForm.value ? (
        <div class='flex items-center gap-2'>
          <Select
            id='field-provider'
            value={d.providerId.value}
            onChange={(e) => {
              a.onProviderChange((e.target as HTMLSelectElement).value);
            }}
            disabled={d.isEdit || status.submitting.value}
            class='w-full'
          >
            <option value=''>Select a provider</option>
            {d.providers.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </Select>
          {!d.isEdit && (
            <Button variant='ghost' onClick={a.onShowProviderForm}>
              + New
            </Button>
          )}
        </div>
      ) : (
        <ProviderFormFields
          name={d.newProviderName.value}
          url={d.newProviderUrl.value}
          apiKey={d.newProviderKey.value}
          providerType={d.newProviderType.value}
          error={d.providerFormError.value}
          submitting={d.providerSubmitting.value}
          mode='create'
          onSave={a.onCreateProvider}
          onCancel={a.onCancelProviderForm}
          onNameChange={a.onNewProviderNameChange}
          onUrlChange={a.onNewProviderUrlChange}
          onApiKeyChange={a.onNewProviderKeyChange}
          onProviderTypeChange={a.onNewProviderTypeChange}
        />
      )}
    </div>
  );
};
