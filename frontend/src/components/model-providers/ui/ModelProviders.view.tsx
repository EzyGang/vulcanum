import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { CatalogProvider, ModelProviderConfig } from '../../../types/modelProviders';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
import { Select } from '../../shared/ui/Select.view';
import { Table } from '../../shared/ui/Table.view';
import { TextArea } from '../../shared/ui/TextArea.view';

interface ModelProvidersViewProps {
  data: {
    catalogProviders: CatalogProvider[];
    providers: ModelProviderConfig[];
    selectedCatalogProvider?: CatalogProvider;
    showForm: Signal<boolean>;
    editId: Signal<string | null>;
    providerKey: Signal<string>;
    displayName: Signal<string>;
    advancedOptions: Signal<string>;
    credentials: Signal<Record<string, string>>;
    formError: Signal<string | null>;
    formSubmitting: Signal<boolean>;
    deleteConfirmId: Signal<string | null>;
    deleteError: Signal<string | null>;
  };
  status: {
    loading: boolean;
    catalogLoading: boolean;
    error: ApiError | null;
  };
  actions: {
    onShowCreate: () => void;
    onShowEdit: (provider: ModelProviderConfig) => void;
    onCancelForm: () => void;
    onProviderChange: (value: string) => void;
    onDisplayNameChange: (value: string) => void;
    onAdvancedOptionsChange: (value: string) => void;
    onCredentialChange: (key: string, value: string) => void;
    onSave: (e: Event) => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDelete: (id: string) => void;
  };
}

export const ModelProvidersView = ({
  data,
  status,
  actions
}: ModelProvidersViewProps): JSX.Element => (
  <div class='flex flex-col gap-4'>
    <SectionHeader
      title='Model Providers'
      hint='Connect LLM providers for app-managed agent runtime config.'
      action={
        !data.showForm.value ? (
          <Button
            variant='primary'
            class='shrink-0 whitespace-nowrap px-5'
            onClick={actions.onShowCreate}
            disabled={status.catalogLoading}
          >
            Add Model Provider
          </Button>
        ) : null
      }
    />

    {status.error && <ErrorBanner message={status.error.message} />}
    {data.deleteError.value && <ErrorBanner message={data.deleteError.value} />}
    {status.loading && <div class='text-text-muted text-sm'>Loading model providers...</div>}

    {data.showForm.value && (
      <form
        onSubmit={actions.onSave}
        class='border border-border-base bg-bg-card p-5 flex flex-col gap-4'
      >
        <div class='flex flex-col gap-2'>
          <Label for='model-provider-key'>Provider</Label>
          <Select
            id='model-provider-key'
            value={data.providerKey.value}
            onValueChange={actions.onProviderChange}
            disabled={!!data.editId.value || data.formSubmitting.value}
            placeholder='Select a provider...'
            items={data.catalogProviders.map((provider) => ({
              value: provider.id,
              label: provider.name
            }))}
          />
        </div>

        <div class='flex flex-col gap-2'>
          <Label for='model-provider-display-name'>Display Name</Label>
          <Input
            id='model-provider-display-name'
            value={data.displayName.value}
            onInput={(e) => actions.onDisplayNameChange((e.target as HTMLInputElement).value)}
            disabled={data.formSubmitting.value}
            placeholder='Production Anthropic'
          />
        </div>

        {data.selectedCatalogProvider && (
          <div class='flex flex-col gap-3'>
            <div class='text-text-muted text-xs'>Credential fields from models.dev catalog.</div>
            {data.selectedCatalogProvider.env.map((envName) => (
              <div class='flex flex-col gap-2' key={envName}>
                <Label for={`credential-${envName}`}>{envName}</Label>
                <Input
                  id={`credential-${envName}`}
                  type='password'
                  value={data.credentials.value[envName] ?? ''}
                  onInput={(e) =>
                    actions.onCredentialChange(envName, (e.target as HTMLInputElement).value)
                  }
                  disabled={data.formSubmitting.value}
                />
              </div>
            ))}
          </div>
        )}

        <div class='flex flex-col gap-2'>
          <Label for='model-provider-options'>Advanced Options JSON</Label>
          <TextArea
            id='model-provider-options'
            value={data.advancedOptions.value}
            onInput={(e) =>
              actions.onAdvancedOptionsChange((e.target as HTMLTextAreaElement).value)
            }
            disabled={data.formSubmitting.value}
            rows={5}
          />
        </div>

        {data.formError.value && <ErrorBanner message={data.formError.value} />}

        <div class='flex items-center gap-3'>
          <Button type='submit' variant='primary' disabled={data.formSubmitting.value}>
            {data.formSubmitting.value ? 'Saving...' : data.editId.value ? 'Update' : 'Create'}
          </Button>
          <Button type='button' variant='secondary' onClick={actions.onCancelForm}>
            Cancel
          </Button>
        </div>
      </form>
    )}

    {!status.loading && data.providers.length === 0 && !data.showForm.value && (
      <EmptyState
        title='No model providers connected yet.'
        description='Connect a model provider to select app-managed runtime models for projects.'
      />
    )}

    {!status.loading && data.providers.length > 0 && (
      <Table>
        <Table.Head>
          <Table.HeadCell>Name</Table.HeadCell>
          <Table.HeadCell>Provider</Table.HeadCell>
          <Table.HeadCell>Credential Fields</Table.HeadCell>
          <Table.HeadCell>Actions</Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {data.providers.map((provider) => (
            <Table.Row key={provider.id}>
              <Table.Cell>
                <span class='text-text-primary text-sm'>
                  {provider.displayName || provider.providerKey}
                </span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>{provider.providerKey}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>
                  {Object.keys(provider.credentials ?? {}).join(', ') || '—'}
                </span>
              </Table.Cell>
              <Table.Cell>
                <ConfirmDelete
                  itemId={provider.id}
                  deletingId={data.deleteConfirmId}
                  onConfirm={actions.onConfirmDelete}
                  onDelete={actions.onDelete}
                  onCancel={actions.onCancelDelete}
                  editActions={
                    <Button variant='ghost' onClick={() => actions.onShowEdit(provider)}>
                      Edit
                    </Button>
                  }
                />
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table>
    )}
  </div>
);
