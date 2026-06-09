import { useLocation } from 'wouter-preact';
import { listProjects } from '../../../services/projects/projects.service';
import { listProviders } from '../../../services/providers/providers.service';
import { listWorkers } from '../../../services/workers/workers.service';
import type { Worker } from '../../../types/workers';
import { useApiQuery } from '../../../utils/api/query/hooks';
import { formatRelativeTime } from '../../../utils/format';
import { useGitHubApp } from '../../github/hooks/useGitHubApp.hook';

interface DashboardStats {
  enabledProjects: number;
  idleWorkers: number;
  busyWorkers: number;
  disconnectedWorkers: number;
}

const countByStatus = (workers: Worker[], status: string): number =>
  workers.filter((w) => w.status === status).length;

export const useDashboard = () => {
  const [, setLocation] = useLocation();

  const {
    data: workers,
    isLoading: workersLoading,
    error: workersError
  } = useApiQuery(['workers'], () => listWorkers());

  const { data: projects, isLoading: projectsLoading } = useApiQuery(['projects'], () =>
    listProjects()
  );

  const {
    data: providers,
    isLoading: providersLoading,
    error: providersError
  } = useApiQuery(['providers'], () => listProviders());

  const github = useGitHubApp();

  const rawWorkers = workers ?? [];
  const stats: DashboardStats = {
    enabledProjects: (projects ?? []).filter((p) => p.enabled).length,
    idleWorkers: countByStatus(rawWorkers, 'idle'),
    busyWorkers: countByStatus(rawWorkers, 'busy'),
    disconnectedWorkers: countByStatus(rawWorkers, 'disconnected')
  };

  const formattedWorkers = rawWorkers.map((w) => ({
    id: w.id,
    name: w.name,
    status: w.status,
    lastSeen: formatRelativeTime(w.lastSeen)
  }));

  const loading = workersLoading || projectsLoading || providersLoading;
  const anyError = workersError ?? providersError ?? null;

  return {
    data: {
      stats,
      workers: formattedWorkers,
      projects: projects ?? [],
      providers: providers ?? [],
      githubInstallation: github.installation ?? null,
      githubLoading: github.installationLoading
    },
    status: {
      loading,
      error: anyError
    },
    actions: {
      goToSettings: () => setLocation('/settings'),
      goToWorkers: () => setLocation('/workers'),
      goToRuns: () => setLocation('/runs'),
      goToNewProject: () => setLocation('/projects/connect')
    }
  };
};
