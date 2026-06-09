import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Table } from '../../shared/ui/Table.view';
import { ProviderFormFields } from './ProviderFormFields.view';

const PROVIDER_TYPE_LABELS: Record<string, string> = {
  kaneo: 'Kaneo'
};

interface ProvidersViewProps {
  data: {
    providers: IntegrationProvider[];
    deleteConfirmId: Signal<string | null>;
    deleteError: Signal<string | null>;
    showForm: Signal<boolean>;
    editId: Signal<string | null>;
    name: string;
    url: string;
    apiKey: string;
    providerType: string;
    formError: string | null;
    formSubmitting: boolean;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
  };
  actions: {
    onShowCreate: () => void;
    onShowEdit: (provider: IntegrationProvider) => void;
    onCancelForm: () => void;
    onSave: (e: Event) => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDelete: (id: string) => void;
    onNameChange: (value: string) => void;
    onUrlChange: (value: string) => void;
    onApiKeyChange: (value: string) => void;
    onProviderTypeChange: (value: string) => void;
  };
}

export const ProvidersView = ({
  data: {
    providers,
    deleteConfirmId,
    deleteError,
    showForm,
    editId,
    name,
    url,
    apiKey,
    providerType,
    formError,
    formSubmitting
  },
  status: { loading, error },
  actions: {
    onShowCreate,
    onShowEdit,
    onCancelForm,
    onSave,
    onConfirmDelete,
    onCancelDelete,
    onDelete,
    onNameChange,
    onUrlChange,
    onApiKeyChange,
    onProviderTypeChange
  }
}: ProvidersViewProps): JSX.Element => (
  <div class='flex flex-col gap-4'>
    <div class='flex items-center justify-between'>
      <h3 class='text-base font-semibold text-text-secondary uppercase tracking-wide'>
        All Providers
      </h3>
      {!showForm.value && (
        <Button variant='primary' onClick={onShowCreate}>
          Add Provider
        </Button>
      )}
    </div>

    {error && <ErrorBanner message={error.message} />}

    {deleteError.value && <ErrorBanner message={deleteError.value} />}

    {loading && <div class='text-text-muted text-sm'>Loading providers...</div>}

    {!loading && !error && providers.length === 0 && !showForm.value && (
      <EmptyState
        title='No providers configured yet.'
        description='Add a provider to connect to your projects.'
      />
    )}

    {showForm.value && (
      <ProviderFormFields
        name={name}
        url={url}
        apiKey={apiKey}
        providerType={providerType}
        error={formError}
        submitting={formSubmitting}
        mode={editId.value ? 'edit' : 'create'}
        onSave={onSave}
        onCancel={onCancelForm}
        onNameChange={onNameChange}
        onUrlChange={onUrlChange}
        onApiKeyChange={onApiKeyChange}
        onProviderTypeChange={onProviderTypeChange}
      />
    )}

    {!loading && providers.length > 0 && (
      <Table>
        <Table.Head>
          <Table.HeadCell>Name</Table.HeadCell>
          <Table.HeadCell>Type</Table.HeadCell>
          <Table.HeadCell>Instance URL</Table.HeadCell>
          <Table.HeadCell>Created</Table.HeadCell>
          <Table.HeadCell>Actions</Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {providers.map((provider) => (
            <Table.Row key={provider.id}>
              <Table.Cell>
                <span class='text-text-primary text-sm'>{provider.name}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm'>
                  {PROVIDER_TYPE_LABELS[provider.providerType] ?? provider.providerType}
                </span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>{provider.instanceUrl}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm'>{provider.createdAt}</span>
              </Table.Cell>
              <Table.Cell>
                <ConfirmDelete
                  itemId={provider.id}
                  deletingId={deleteConfirmId}
                  onConfirm={onConfirmDelete}
                  onDelete={onDelete}
                  onCancel={onCancelDelete}
                  editActions={
                    <Button variant='ghost' onClick={() => onShowEdit(provider)}>
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
