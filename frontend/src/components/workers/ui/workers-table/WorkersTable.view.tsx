import type { Signal } from '@preact/signals';
import { IconBan, IconRefresh } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { UpdateWorkerStatusRequest } from '../../../../types/workers';
import { ActionIconButton } from '../../../shared/ui/ActionIconButton.view';
import { ConfirmDelete } from '../../../shared/ui/ConfirmDelete.view';
import { ProgressBar } from '../../../shared/ui/ProgressBar.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import type { FormattedWorker } from '../../hooks/useWorkers.hook';

interface WorkersTableProps {
  workers: FormattedWorker[];
  deletingId: Signal<string | null>;
  actions: {
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDeleteWorker: (id: string) => void;
    onUpdateStatus: (id: string, status: UpdateWorkerStatusRequest['status']) => void;
  };
}

export const WorkersTable = ({
  workers,
  deletingId,
  actions: { onConfirmDelete, onCancelDelete, onDeleteWorker, onUpdateStatus }
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
            <span class='text-text-primary text-sm font-mono'>{worker.name}</span>
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
