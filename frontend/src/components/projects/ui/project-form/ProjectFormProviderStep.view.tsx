import type { JSX } from 'preact';
import { ProviderFormFields } from '../../../providers/ui/ProviderFormFields.view';
import { Button } from '../../../shared/ui/Button.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';
import { useProjectFormProviderContext } from '../../context/ProjectFormProviderContext';

export const ProjectFormProviderStep = (): JSX.Element => {
  const m = useProjectFormMetaContext();
  const p = useProjectFormProviderContext();

  return (
    <div class='flex flex-col gap-2'>
      <Label for='field-provider'>Provider</Label>
      {p.providers.length > 0 && !p.showProviderForm.value ? (
        <div class='flex items-center gap-2'>
          <Select
            id='field-provider'
            value={p.providerId.value}
            onChange={(e) => {
              p.onProviderChange((e.target as HTMLSelectElement).value);
            }}
            disabled={m.isEdit || m.submitting.value}
            class='w-full'
          >
            <option value=''>Select a provider</option>
            {p.providers.map((prov) => (
              <option key={prov.id} value={prov.id}>
                {prov.name}
              </option>
            ))}
          </Select>
          {!m.isEdit && (
            <Button variant='ghost' onClick={p.onShowProviderForm}>
              + New
            </Button>
          )}
        </div>
      ) : (
        <ProviderFormFields
          name={p.newProviderName.value}
          url={p.newProviderUrl.value}
          apiKey={p.newProviderKey.value}
          providerType={p.newProviderType.value}
          error={p.providerFormError.value}
          submitting={p.providerSubmitting.value}
          mode='create'
          onSave={p.onCreateProvider}
          onCancel={p.onCancelProviderForm}
          onNameChange={p.onNewProviderNameChange}
          onUrlChange={p.onNewProviderUrlChange}
          onApiKeyChange={p.onNewProviderKeyChange}
          onProviderTypeChange={p.onNewProviderTypeChange}
        />
      )}
    </div>
  );
};
