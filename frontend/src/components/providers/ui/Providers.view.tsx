import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
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
    <SectionHeader
      title='Task Tracker Providers'
      hint='Connect task tracker providers for project integrations.'
      action={
        !showForm.value ? (
          <Button variant='primary' class='shrink-0 whitespace-nowrap px-5' onClick={onShowCreate}>
            Add Task Tracker
          </Button>
        ) : null
      }
    />

    {error && <ErrorBanner message={error.message} />}

    {deleteError.value && <ErrorBanner message={deleteError.value} />}

    {loading && <div class='text-text-muted text-sm'>Loading task tracker providers...</div>}

    {!loading && !error && providers.length === 0 && !showForm.value && (
      <EmptyState
        title='No task tracker providers configured yet.'
        description='Add a task tracker provider to connect your projects.'
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
