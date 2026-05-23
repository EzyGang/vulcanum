import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { formatDuration, formatRelativeTime } from '../../../utils/format';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';

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
    runsLoading: boolean;
    workersLoading: boolean;
    statsLoading: boolean;
    runsError: ApiError | null;
    workersError: ApiError | null;
    statsError: ApiError | null;
  };
}

const SkeletonStatCard = ({ label }: { label: string }): JSX.Element => (
  <div class='flex flex-col gap-2 bg-bg-card border border-border-base p-5'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <div class='h-8 w-12 bg-bg-hover animate-pulse' />
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

const StatCard = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <div class='flex flex-col gap-1 bg-bg-card border border-border-base p-5'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <span class='text-text-primary text-2xl font-semibold font-mono'>{value}</span>
  </div>
);

const StatsGrid = ({
  stats,
  loading,
  statsError,
  workersError
}: {
  stats: StatsData | null;
  loading: boolean;
  statsError: ApiError | null;
  workersError: ApiError | null;
}): JSX.Element => (
  <section class='flex flex-col gap-4'>
    <div class='grid grid-cols-4 gap-4'>
      {loading && !stats ? (
        <>
          <SkeletonStatCard label='Enabled Projects' />
          <SkeletonStatCard label='Idle Workers' />
          <SkeletonStatCard label='Busy Workers' />
          <SkeletonStatCard label='Disconnected Workers' />
        </>
      ) : (
        <>
          <StatCard label='Enabled Projects' value={stats?.enabledProjects ?? 0} />
          <StatCard label='Idle Workers' value={stats?.idleWorkers ?? 0} />
          <StatCard label='Busy Workers' value={stats?.busyWorkers ?? 0} />
          <StatCard label='Disconnected Workers' value={stats?.disconnectedWorkers ?? 0} />
        </>
      )}
    </div>

    {statsError && (
      <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
        {statsError.message}
      </div>
    )}

    {workersError && (
      <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
        {workersError.message}
      </div>
    )}
  </section>
);

export const DashboardView = ({
  data: { runs, stats },
  status: { runsLoading, workersLoading, statsLoading, runsError, workersError, statsError }
}: DashboardViewProps): JSX.Element => {
  const allDataMissing = !stats && runs.length === 0;
  const anyLoading = runsLoading || workersLoading || statsLoading;

  return (
    <div class='flex flex-col gap-8'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Dashboard</h2>

      {anyLoading && allDataMissing && <div class='text-text-muted text-sm'>Loading...</div>}

      <StatsGrid
        stats={stats}
        loading={statsLoading || workersLoading}
        statsError={statsError}
        workersError={workersError}
      />

      <section class='flex flex-col gap-4'>
        <h3 class='text-md font-semibold text-text-primary uppercase tracking-wide'>
          Recent Work Runs
        </h3>

        {runsError && (
          <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
            {runsError.message}
          </div>
        )}

        {runs.length === 0 && !runsLoading && (
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
                    <td class='px-5 py-3'>
                      <StatusBadge status={run.status} />
                    </td>
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
};
