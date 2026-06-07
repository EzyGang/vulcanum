import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
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
    onUpdateStatus: (id: string, status: string) => void;
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
      <Table.HeadCell>Last Seen</Table.HeadCell>
      <Table.HeadCell>Load</Table.HeadCell>
      <Table.HeadCell>Actions</Table.HeadCell>
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
          <Table.Cell>
            <span class='text-text-secondary text-sm'>{worker.lastSeen}</span>
          </Table.Cell>
          <Table.Cell>
            <ProgressBar value={worker.activeJobs} max={worker.maxConcurrentJobs} showFraction />
          </Table.Cell>
          <Table.Cell>
            <div class='flex items-center gap-2'>
              {worker.status === 'unhealthy' && (
                <Button variant='ghost' onClick={() => onUpdateStatus(worker.id, 'idle')}>
                  Re-enable
                </Button>
              )}
              {(worker.status === 'idle' || worker.status === 'busy') && (
                <Button variant='ghost' onClick={() => onUpdateStatus(worker.id, 'unhealthy')}>
                  Disable
                </Button>
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
