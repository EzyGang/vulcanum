import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { UpdateWorkerStatusRequest } from '../../../types/workers';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { Card } from '../../shared/ui/Card.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { FormattedWorker } from '../hooks/useWorkers.hook';
import { WorkersTable } from './workers-table/WorkersTable.view';

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
        <WorkersTable
          workers={workers}
          deletingId={deletingId}
          actions={{ onConfirmDelete, onCancelDelete, onDeleteWorker, onUpdateStatus }}
        />
      )}
    </section>
  </div>
);
