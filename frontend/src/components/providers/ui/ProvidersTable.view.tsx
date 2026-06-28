import type { Signal } from '@preact/signals';
import { IconPencil } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import { ActionIconButton } from '../../shared/ui/ActionIconButton.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { Table } from '../../shared/ui/Table.view';

const PROVIDER_TYPE_LABELS: Record<string, string> = {
  kaneo: 'Kaneo'
};

type ProviderRow = IntegrationProvider & { formattedCreatedAt: string };

interface ProvidersTableProps {
  providers: ProviderRow[];
  deleteConfirmId: Signal<string | null>;
  onShowEdit: (provider: IntegrationProvider) => void;
  onConfirmDelete: (id: string) => void;
  onDelete: (id: string) => void;
  onCancelDelete: () => void;
}

export const ProvidersTable = ({
  providers,
  deleteConfirmId,
  onShowEdit,
  onConfirmDelete,
  onDelete,
  onCancelDelete
}: ProvidersTableProps): JSX.Element => (
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
            <span class='text-text-secondary text-sm'>{provider.formattedCreatedAt}</span>
          </Table.Cell>
          <Table.Cell>
            <ConfirmDelete
              itemId={provider.id}
              deletingId={deleteConfirmId}
              onConfirm={onConfirmDelete}
              onDelete={onDelete}
              onCancel={onCancelDelete}
              editActions={
                <ActionIconButton label='Edit provider' onClick={() => onShowEdit(provider)}>
                  <IconPencil size={16} stroke={1.75} aria-hidden='true' />
                </ActionIconButton>
              }
            />
          </Table.Cell>
        </Table.Row>
      ))}
    </Table.Body>
  </Table>
);
