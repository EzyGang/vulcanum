import type { Signal } from '@preact/signals';
import { IconBan, IconRefresh } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { UpdateWorkerStatusRequest } from '../../../../types/workers';
import { ActionIconButton } from '../../../shared/ui/ActionIconButton.view';
import { Button } from '../../../shared/ui/Button.view';
import { ConfirmDelete } from '../../../shared/ui/ConfirmDelete.view';
import { ProgressBar } from '../../../shared/ui/ProgressBar.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import type { FormattedWorker } from '../../hooks/useWorkers.hook';

interface WorkersTableProps {
  workers: FormattedWorker[];
  deletingId: Signal<string | null>;
  renameLoading: boolean;
  actions: {
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDeleteWorker: (id: string) => void;
    onUpdateStatus: (id: string, status: UpdateWorkerStatusRequest['status']) => void;
    onRenameWorker: (id: string, name: string) => void;
  };
}

export const WorkersTable = ({
  workers,
  deletingId,
  renameLoading,
  actions: { onConfirmDelete, onCancelDelete, onDeleteWorker, onUpdateStatus, onRenameWorker }
}: WorkersTableProps): JSX.Element => (
  <Table>
    <Table.Head>
      <Table.HeadCell>Name</Table.HeadCell>
      <Table.HeadCell>Status</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Last Seen</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Load</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Actions</Table.HeadCell>
    </Table.Head>
    <Table.Body>
      {workers.map((worker) => (
        <Table.Row key={worker.id}>
          <Table.Cell>
            <form
              class='flex items-center gap-2'
              onSubmit={(event) => {
                event.preventDefault();
                const formData = new FormData(event.currentTarget);
                onRenameWorker(worker.id, String(formData.get('name') ?? ''));
              }}
            >
              <label class='sr-only' for={`worker-name-${worker.id}`}>
                Worker name
              </label>
              <input
                id={`worker-name-${worker.id}`}
                name='name'
                defaultValue={worker.name}
                disabled={renameLoading}
                required
                class='min-w-0 bg-bg-input border border-border-base px-2 py-1 text-text-primary text-sm font-mono outline-none focus:border-border-focus'
              />
              <Button type='submit' variant='ghost' class='px-1 py-1' disabled={renameLoading}>
                Rename
              </Button>
            </form>
          </Table.Cell>
          <Table.Cell>
            <StatusBadge status={worker.status} />
          </Table.Cell>
          <Table.Cell class='hidden md:table-cell'>
            <span class='text-text-secondary text-sm'>{worker.lastSeen}</span>
          </Table.Cell>
          <Table.Cell class='hidden md:table-cell'>
            <ProgressBar value={worker.activeJobs} max={worker.maxConcurrentJobs} showFraction />
          </Table.Cell>
          <Table.Cell class='hidden md:table-cell'>
            <div class='flex items-center gap-2'>
              {worker.status === 'unhealthy' && (
                <ActionIconButton
                  label='Re-enable worker'
                  variant='success'
                  onClick={() => onUpdateStatus(worker.id, 'idle')}
                >
                  <IconRefresh size={16} stroke={1.75} aria-hidden='true' />
                </ActionIconButton>
              )}
              {(worker.status === 'idle' || worker.status === 'busy') && (
                <ActionIconButton
                  label='Disable worker'
                  onClick={() => onUpdateStatus(worker.id, 'unhealthy')}
                >
                  <IconBan size={16} stroke={1.75} aria-hidden='true' />
                </ActionIconButton>
              )}
              <ConfirmDelete
                itemId={worker.id}
                deletingId={deletingId}
                onConfirm={onConfirmDelete}
                onDelete={onDeleteWorker}
                onCancel={onCancelDelete}
              />
            </div>
          </Table.Cell>
        </Table.Row>
      ))}
    </Table.Body>
  </Table>
);
