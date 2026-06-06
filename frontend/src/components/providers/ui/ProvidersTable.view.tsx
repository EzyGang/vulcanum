import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { Table } from '../../shared/ui/Table.view';

const PROVIDER_TYPE_LABELS: Record<string, string> = {
  kaneo: 'Kaneo'
};

interface ProvidersTableProps {
  providers: IntegrationProvider[];
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
);
