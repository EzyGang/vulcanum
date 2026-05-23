import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem, WorkRunStatus } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { formatDuration, formatRelativeTime } from '../../../utils/format';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';

const STATUS_OPTIONS: { value: WorkRunStatus | ''; label: string }[] = [
  { value: '', label: 'All' },
  { value: 'pending', label: 'Pending' },
  { value: 'dispatched', label: 'Dispatched' },
  { value: 'running', label: 'Running' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
  { value: 'stalled', label: 'Stalled' }
];

interface RunsViewProps {
  data: {
    runs: WorkRunListItem[];
  };
  status: {
    loading: boolean;
    error: ApiError | null;
    statusFilter: Signal<WorkRunStatus | undefined>;
    page: Signal<number>;
    hasNextPage: boolean;
    hasPrevPage: boolean;
  };
  actions: {
    setStatusFilter: (status: WorkRunStatus | undefined) => void;
    nextPage: () => void;
    prevPage: () => void;
  };
}

export const RunsView = ({
  data: { runs },
  status: { loading, error, statusFilter, page, hasNextPage, hasPrevPage },
  actions: { setStatusFilter, nextPage, prevPage }
}: RunsViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <div class='flex items-center justify-between'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Work Runs</h2>

      <select
        value={statusFilter.value ?? ''}
        onChange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          setStatusFilter(val ? (val as WorkRunStatus) : undefined);
        }}
        class='bg-bg-input border border-border-base text-text-primary text-sm px-3 py-2 focus:outline-none focus:border-border-focus'
      >
        {STATUS_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>

    {error && (
      <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
        {error.message}
      </div>
    )}

    {loading && <div class='text-text-muted text-sm'>Loading runs...</div>}

    {!loading && !error && runs.length === 0 && (
      <div class='flex flex-col items-center gap-2 bg-bg-card border border-border-base p-12'>
        <p class='text-text-muted text-sm'>No work runs found.</p>
      </div>
    )}

    {runs.length > 0 && (
      <div class='overflow-x-auto'>
        <table class='w-full border-collapse'>
          <thead>
            <tr class='border-b border-border-base'>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                Task
              </th>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                Status
              </th>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                Worker
              </th>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                Duration
              </th>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                PR
              </th>
              <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                Created
              </th>
            </tr>
          </thead>
          <tbody>
            {runs.map((run) => (
              <tr key={run.id} class='border-b border-border-base'>
                <td class='px-5 py-3'>
                  <span class='text-text-primary text-sm font-mono'>{run.externalTaskRef}</span>
                </td>
                <td class='px-5 py-3'>
                  <StatusBadge status={run.status} />
                </td>
                <td class='px-5 py-3'>
                  <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
                </td>
                <td class='px-5 py-3'>
                  <span class='text-text-secondary text-sm font-mono'>
                    {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
                  </span>
                </td>
                <td class='px-5 py-3'>
                  {run.resultPrUrl ? (
                    <a
                      href={run.resultPrUrl}
                      target='_blank'
                      rel='noopener noreferrer'
                      class='text-accent text-sm hover:underline'
                    >
                      PR
                    </a>
                  ) : (
                    <span class='text-text-muted text-sm'>—</span>
                  )}
                </td>
                <td class='px-5 py-3'>
                  <span class='text-text-secondary text-sm'>
                    {formatRelativeTime(run.createdAt)}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        <div class='flex items-center justify-between pt-4'>
          <button
            type='button'
            onClick={prevPage}
            disabled={!hasPrevPage || loading}
            class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors disabled:opacity-30'
          >
            Previous
          </button>
          <span class='text-text-muted text-sm'>Page {page.value + 1}</span>
          <button
            type='button'
            onClick={nextPage}
            disabled={!hasNextPage || loading}
            class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors disabled:opacity-30'
          >
            Next
          </button>
        </div>
      </div>
    )}
  </div>
);
