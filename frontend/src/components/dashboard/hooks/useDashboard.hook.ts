import { getProjectsStats } from '../../../services/projects/projects.service';
import { listRuns } from '../../../services/runs/runs.service';
import { listWorkers } from '../../../services/workers/workers.service';
import type { Worker } from '../../../types/workers';
import { useApiQuery } from '../../../utils/api/query/hooks';

interface DashboardStats {
  enabledProjects: number;
  idleWorkers: number;
  busyWorkers: number;
  disconnectedWorkers: number;
}

const countByStatus = (workers: Worker[], status: string): number =>
  workers.filter((w) => w.status === status).length;

export const useDashboard = () => {
  const {
    data: runs,
    isLoading: runsLoading,
    error: runsError
  } = useApiQuery(['runs', 'recent'], () => listRuns({ limit: 10 }));

  const {
    data: workers,
    isLoading: workersLoading,
    error: workersError
  } = useApiQuery(['workers'], () => listWorkers());

  const {
    data: stats,
    isLoading: statsLoading,
    error: statsError
  } = useApiQuery(['projectsStats'], () => getProjectsStats());

  const loading = runsLoading || workersLoading || statsLoading;
  const error = runsError || workersError || statsError;

  const dashboardStats: DashboardStats | null =
    stats && workers
      ? {
          enabledProjects: stats.enabledCount,
          idleWorkers: countByStatus(workers, 'idle'),
          busyWorkers: countByStatus(workers, 'busy'),
          disconnectedWorkers: countByStatus(workers, 'disconnected')
        }
      : null;

  return {
    data: {
      runs: runs ?? [],
      stats: dashboardStats
    },
    status: {
      loading,
      error
    }
  };
};
