import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { UpdateWorkerStatusRequest } from '../../../types/workers';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { Card } from '../../shared/ui/Card.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { ProgressBar } from '../../shared/ui/ProgressBar.view';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { Table } from '../../shared/ui/Table.view';
import type { FormattedWorker } from '../hooks/useWorkers.hook';

interface WorkersViewProps {
  data: {
    workers: FormattedWorker[];
    code: string | null;
    countdown: Signal<string>;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
    generateLoading: boolean;
    deletingId: Signal<string | null>;
    deleteError: Signal<string | null>;
    updateStatusError: ApiError | null;
  };
  actions: {
    onGenerateCode: () => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDeleteWorker: (id: string) => void;
    onUpdateStatus: (id: string, status: UpdateWorkerStatusRequest['status']) => void;
  };
}

export const WorkersView = ({
  data: { workers, code, countdown },
  status: { loading, error, generateLoading, deletingId, deleteError, updateStatusError },
  actions: { onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker, onUpdateStatus }
}: WorkersViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <section class='flex flex-col gap-4'>
      <div class='flex items-center justify-between'>
        <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
          Registration Codes
        </h2>
        <Button variant='primary' onClick={onGenerateCode} disabled={generateLoading}>
          {generateLoading ? 'Generating...' : 'Generate Code'}
        </Button>
      </div>

      {code && (
        <Card class='flex flex-col gap-2'>
          <div class='flex items-center gap-4'>
            <span class='text-text-muted text-sm uppercase tracking-wider'>Code:</span>
            <code class='text-accent font-mono text-lg tracking-widest'>{code}</code>
          </div>
          {countdown.value && <span class='text-text-muted text-sm'>{countdown.value}</span>}
        </Card>
      )}
    </section>

    <section class='flex flex-col gap-4'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Workers</h2>

      {error && <ErrorBanner message={error.message} />}

      {deleteError.value && <ErrorBanner message={deleteError.value} />}

      {updateStatusError && <ErrorBanner message={updateStatusError.message} />}

      {loading && <div class='text-text-muted text-sm'>Loading workers...</div>}

      {!loading && !error && workers.length === 0 && (
        <EmptyState
          title='No workers registered yet.'
          description='Generate a registration code above and use it to connect a worker daemon.'
        />
      )}

      {!loading && workers.length > 0 && (
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
                  <ProgressBar
                    value={worker.activeJobs}
                    max={worker.maxConcurrentJobs}
                    showFraction
                  />
                </Table.Cell>
                <Table.Cell class='hidden md:table-cell'>
                  <div class='flex items-center gap-2'>
                    {worker.status === 'unhealthy' && (
                      <Button variant='ghost' onClick={() => onUpdateStatus(worker.id, 'idle')}>
                        Re-enable
                      </Button>
                    )}
                    {(worker.status === 'idle' || worker.status === 'busy') && (
                      <Button
                        variant='ghost'
                        onClick={() => onUpdateStatus(worker.id, 'unhealthy')}
                      >
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
      )}
    </section>
  </div>
);
