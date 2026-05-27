import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ApiError } from '../../../utils/api/client';
import { ProgressBar } from '../../shared/ui/ProgressBar.view';
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
  };
  actions: {
    onGenerateCode: () => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDeleteWorker: (id: string) => void;
  };
}

const statusBadge = (status: string): JSX.Element => {
  const colors: Record<string, string> = {
    idle: 'text-success bg-success-bg border-success-border',
    busy: 'text-warning bg-warning-bg border-warning-border',
    disconnected: 'text-error bg-error-bg border-error-border'
  };

  return (
    <span
      class={`text-xs uppercase tracking-wider px-2 py-0.5 border ${colors[status] ?? 'text-text-muted bg-bg-hover border-border-base'}`}
    >
      {status}
    </span>
  );
};

export const WorkersView = ({
  data: { workers, code, countdown },
  status: { loading, error, generateLoading, deletingId, deleteError },
  actions: { onGenerateCode, onConfirmDelete, onCancelDelete, onDeleteWorker }
}: WorkersViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <section class='flex flex-col gap-4'>
      <div class='flex items-center justify-between'>
        <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
          Registration Codes
        </h2>
        <button
          type='button'
          onClick={onGenerateCode}
          disabled={generateLoading}
          class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider
                 px-4 py-3 hover:opacity-90 transition-opacity disabled:opacity-50'
        >
          {generateLoading ? 'Generating...' : 'Generate Code'}
        </button>
      </div>

      {code && (
        <div class='flex flex-col gap-2 bg-bg-card border border-border-base p-5'>
          <div class='flex items-center gap-4'>
            <span class='text-text-muted text-sm uppercase tracking-wider'>Code:</span>
            <code class='text-accent font-mono text-lg tracking-widest'>{code}</code>
          </div>
          {countdown.value && <span class='text-text-muted text-sm'>{countdown.value}</span>}
        </div>
      )}
    </section>

    <section class='flex flex-col gap-4'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Workers</h2>

      {error && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {error.message}
        </div>
      )}

      {deleteError.value && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {deleteError.value}
        </div>
      )}

      {loading && <div class='text-text-muted text-sm'>Loading workers...</div>}

      {!loading && !error && workers.length === 0 && (
        <div class='flex flex-col items-center gap-4 bg-bg-card border border-border-base p-12'>
          <p class='text-text-muted text-sm'>No workers registered yet.</p>
          <p class='text-text-muted text-xs'>
            Generate a registration code above and use it to connect a worker daemon.
          </p>
        </div>
      )}

      {!loading && workers.length > 0 && (
        <div class='overflow-x-auto'>
          <table class='w-full border-collapse'>
            <thead>
              <tr class='border-b border-border-base'>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Name
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Status
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Last Seen
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Load
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {workers.map((worker) => (
                <tr key={worker.id} class='border-b border-border-base'>
                  <td class='px-5 py-3'>
                    <span class='text-text-primary text-sm font-mono'>{worker.name}</span>
                  </td>
                  <td class='px-5 py-3'>{statusBadge(worker.status)}</td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm'>{worker.lastSeen}</span>
                  </td>
                  <td class='px-5 py-3'>
                    <ProgressBar
                      value={worker.activeJobs}
                      max={worker.maxConcurrentJobs}
                      showFraction
                    />
                  </td>
                  <td class='px-5 py-3'>
                    {deletingId.value === worker.id ? (
                      <div class='flex items-center gap-2'>
                        <span class='text-text-muted text-xs'>Confirm?</span>
                        <button
                          type='button'
                          onClick={() => onDeleteWorker(worker.id)}
                          class='text-error text-xs uppercase tracking-wider hover:opacity-80 transition-opacity'
                        >
                          Delete
                        </button>
                        <button
                          type='button'
                          onClick={onCancelDelete}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors'
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <button
                        type='button'
                        onClick={() => onConfirmDelete(worker.id)}
                        class='text-text-muted text-xs uppercase tracking-wider hover:text-error transition-colors'
                      >
                        Delete
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  </div>
);
