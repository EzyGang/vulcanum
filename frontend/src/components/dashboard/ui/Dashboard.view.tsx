import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { formatDuration, formatRelativeTime } from '../../../utils/format';

interface StatsData {
  enabledProjects: number;
  idleWorkers: number;
  busyWorkers: number;
  disconnectedWorkers: number;
}

interface DashboardViewProps {
  data: {
    runs: WorkRunListItem[];
    stats: StatsData | null;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
  };
}

const statusBadge = (status: string): JSX.Element => {
  const colors: Record<string, string> = {
    pending: 'text-text-muted bg-bg-hover border-border-base',
    dispatched: 'text-accent-secondary bg-warning-bg border-warning-border',
    running: 'text-accent bg-success-bg border-success-border',
    completed: 'text-success bg-success-bg border-success-border',
    failed: 'text-error bg-error-bg border-error-border',
    stalled: 'text-warning bg-warning-bg border-warning-border'
  };

  return (
    <span
      class={`text-xs uppercase tracking-wider px-2 py-0.5 border ${colors[status] ?? 'text-text-muted bg-bg-hover border-border-base'}`}
    >
      {status}
    </span>
  );
};

const StatCard = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <div class='flex flex-col gap-1 bg-bg-card border border-border-base p-5'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <span class='text-text-primary text-2xl font-semibold font-mono'>{value}</span>
  </div>
);

const TimeCell = ({ dateStr }: { dateStr: string }): JSX.Element => (
  <span class='text-text-secondary text-sm'>{formatRelativeTime(dateStr)}</span>
);

const DurationCell = ({ ms }: { ms: number | null }): JSX.Element => (
  <span class='text-text-secondary text-sm font-mono'>
    {ms !== null ? formatDuration(ms) : '—'}
  </span>
);

export const DashboardView = ({
  data: { runs, stats },
  status: { loading, error }
}: DashboardViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Dashboard</h2>

    {error && (
      <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
        {error.message}
      </div>
    )}

    {loading && <div class='text-text-muted text-sm'>Loading...</div>}

    {stats && (
      <div class='grid grid-cols-4 gap-4'>
        <StatCard label='Enabled Projects' value={stats.enabledProjects} />
        <StatCard label='Idle Workers' value={stats.idleWorkers} />
        <StatCard label='Busy Workers' value={stats.busyWorkers} />
        <StatCard label='Disconnected Workers' value={stats.disconnectedWorkers} />
      </div>
    )}

    <section class='flex flex-col gap-4'>
      <h3 class='text-md font-semibold text-text-primary uppercase tracking-wide'>
        Recent Work Runs
      </h3>

      {runs.length === 0 && !loading && (
        <div class='flex flex-col items-center gap-2 bg-bg-card border border-border-base p-8'>
          <p class='text-text-muted text-sm'>No work runs yet.</p>
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
                  <td class='px-5 py-3'>{statusBadge(run.status)}</td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
                  </td>
                  <td class='px-5 py-3'>
                    <DurationCell ms={run.durationMs} />
                  </td>
                  <td class='px-5 py-3'>
                    <TimeCell dateStr={run.createdAt} />
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
