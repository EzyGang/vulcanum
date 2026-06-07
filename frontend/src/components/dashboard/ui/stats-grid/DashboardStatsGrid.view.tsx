import type { JSX } from 'preact';
import { Card } from '../../../shared/ui/Card.view';
import { ErrorBanner } from '../../../shared/ui/ErrorBanner.view';

export interface StatsData {
  enabledProjects: number;
  idleWorkers: number;
  busyWorkers: number;
  disconnectedWorkers: number;
}

interface DashboardStatsGridProps {
  stats: StatsData | null;
  loading: boolean;
  statsError: { message: string } | null;
  workersError: { message: string } | null;
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

export const DashboardStatsGrid = ({
  stats,
  loading,
  statsError,
  workersError
}: DashboardStatsGridProps): JSX.Element => (
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
