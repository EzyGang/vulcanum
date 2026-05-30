import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { formatDuration, formatRelativeTime } from '../../../utils/format';
import { Card } from '../../shared/ui/Card.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { Table } from '../../shared/ui/Table.view';

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
  <Card class='flex flex-col gap-2'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <div class='h-8 w-12 bg-bg-hover animate-pulse' />
  </Card>
);

const StatCard = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <Card class='flex flex-col gap-1'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <span class='text-text-primary text-2xl font-semibold font-mono'>{value}</span>
  </Card>
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

    {statsError && <ErrorBanner message={statsError.message} />}
    {workersError && <ErrorBanner message={workersError.message} />}
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

        {runsError && <ErrorBanner message={runsError.message} />}

        {runs.length === 0 && !runsLoading && <EmptyState title='No work runs yet.' />}

        {runs.length > 0 && (
          <Table>
            <Table.Head>
              <Table.HeadCell>Task</Table.HeadCell>
              <Table.HeadCell>Status</Table.HeadCell>
              <Table.HeadCell>Worker</Table.HeadCell>
              <Table.HeadCell>Duration</Table.HeadCell>
              <Table.HeadCell>Created</Table.HeadCell>
            </Table.Head>
            <Table.Body>
              {runs.map((run) => (
                <Table.Row key={run.id}>
                  <Table.Cell>
                    <span class='text-text-primary text-sm font-mono'>{run.externalTaskRef}</span>
                  </Table.Cell>
                  <Table.Cell>
                    <StatusBadge status={run.status} />
                  </Table.Cell>
                  <Table.Cell>
                    <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
                  </Table.Cell>
                  <Table.Cell>
                    <span class='text-text-secondary text-sm font-mono'>
                      {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
                    </span>
                  </Table.Cell>
                  <Table.Cell>
                    <span class='text-text-secondary text-sm'>
                      {formatRelativeTime(run.createdAt)}
                    </span>
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table>
        )}
      </section>
    </div>
  );
};
