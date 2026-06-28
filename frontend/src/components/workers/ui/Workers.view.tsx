import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { UpdateWorkerStatusRequest } from '../../../types/workers';
import type { ApiError } from '../../../utils/api/client';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { FormattedWorker, WorkerRegistrationCopyTarget } from '../hooks/useWorkers.hook';
import { WorkersRegistrationCodes } from './registration-codes/WorkersRegistrationCodes.view';
import { WorkersTable } from './workers-table/WorkersTable.view';

interface WorkersViewProps {
  data: {
    workers: FormattedWorker[];
    maskedCode: string | null;
    setupCommandPreview: string | null;
    countdown: Signal<string>;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
    generateLoading: boolean;
    deletingId: Signal<string | null>;
    deleteError: Signal<string | null>;
    updateStatusError: ApiError | null;
    copiedTarget: Signal<WorkerRegistrationCopyTarget | null>;
    copyError: Signal<string | null>;
  };
  actions: {
    onGenerateCode: () => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDeleteWorker: (id: string) => void;
    onUpdateStatus: (id: string, status: UpdateWorkerStatusRequest['status']) => void;
    onCopyCode: () => void;
    onCopySetupCommand: () => void;
  };
}

export const WorkersView = ({
  data: { workers, maskedCode, setupCommandPreview, countdown },
  status: {
    loading,
    error,
    generateLoading,
    deletingId,
    deleteError,
    updateStatusError,
    copiedTarget,
    copyError
  },
  actions: {
    onGenerateCode,
    onConfirmDelete,
    onCancelDelete,
    onDeleteWorker,
    onUpdateStatus,
    onCopyCode,
    onCopySetupCommand
  }
}: WorkersViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <WorkersRegistrationCodes
      maskedCode={maskedCode}
      setupCommandPreview={setupCommandPreview}
      countdown={countdown}
      generateLoading={generateLoading}
      copiedTarget={copiedTarget}
      copyError={copyError}
      onGenerateCode={onGenerateCode}
      onCopyCode={onCopyCode}
      onCopySetupCommand={onCopySetupCommand}
    />

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
