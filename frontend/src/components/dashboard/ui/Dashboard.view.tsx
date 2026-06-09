import type { JSX } from 'preact';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { Table } from '../../shared/ui/Table.view';
import { DashboardTableSection } from './DashboardTableSection.view';

interface StatsData {
  enabledProjects: number;
  idleWorkers: number;
  busyWorkers: number;
  disconnectedWorkers: number;
}

interface WorkerSummary {
  id: string;
  name: string;
  status: string;
  lastSeen: string;
}

interface ProjectSummary {
  id: string;
  externalProjectId: string;
  name: string;
  enabled: boolean;
}

interface ProviderSummary {
  id: string;
  name: string;
  providerType: string;
}

interface DashboardViewProps {
  data: {
    stats: StatsData;
    workers: WorkerSummary[];
    projects: ProjectSummary[];
    providers: ProviderSummary[];
    githubInstallation: { accountLogin: string } | null;
    githubLoading: boolean;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
  };
  actions: {
    goToSettings: () => void;
    goToWorkers: () => void;
    goToRuns: () => void;
    goToNewProject: () => void;
  };
}

const StatCard = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <div class='flex flex-col gap-1 bg-bg-card border border-border-base p-3 min-w-0'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <span class='text-text-primary text-xl font-semibold font-mono'>{value}</span>
  </div>
);

const SkeletonStatCard = ({ label }: { label: string }): JSX.Element => (
  <div class='flex flex-col gap-2 bg-bg-card border border-border-base p-3 min-w-0'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <div class='h-7 w-10 bg-bg-hover animate-pulse' />
  </div>
);

export const DashboardView = ({
  data: { stats, workers, projects, providers, githubInstallation, githubLoading },
  status: { loading, error },
  actions: { goToSettings, goToWorkers, goToRuns, goToNewProject }
}: DashboardViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Dashboard</h2>

    {error && <ErrorBanner message={error.message} />}

    {loading && !stats && <div class='text-text-muted text-sm'>Loading...</div>}

    <section class='flex flex-col gap-4'>
      <div class='grid grid-cols-2 sm:grid-cols-4 gap-4'>
        {loading ? (
          <>
            <SkeletonStatCard label='Enabled Projects' />
            <SkeletonStatCard label='Idle Workers' />
            <SkeletonStatCard label='Busy Workers' />
            <SkeletonStatCard label='Disconnected Workers' />
          </>
        ) : (
          <>
            <StatCard label='Enabled Projects' value={stats.enabledProjects} />
            <StatCard label='Idle Workers' value={stats.idleWorkers} />
            <StatCard label='Busy Workers' value={stats.busyWorkers} />
            <StatCard label='Disconnected Workers' value={stats.disconnectedWorkers} />
          </>
        )}
      </div>
    </section>

    <div class='flex items-center gap-3 px-4 py-3 bg-bg-card border border-border-base animate-slide-up'>
      <Button variant='ghost' onClick={goToSettings}>
        Add Provider
      </Button>
      <Button variant='ghost' onClick={goToNewProject}>
        Connect Project
      </Button>
      <Button variant='ghost' onClick={goToRuns}>
        View Runs
      </Button>
      <Button variant='ghost' onClick={goToWorkers}>
        Manage Workers
      </Button>
    </div>

    <div class='grid grid-cols-1 lg:grid-cols-2 gap-6'>
      <DashboardTableSection
        title='Workers'
        emptyMessage='No workers registered.'
        isEmpty={workers.length === 0}
      >
        <Table>
          <Table.Head>
            <Table.HeadCell>Name</Table.HeadCell>
            <Table.HeadCell>Status</Table.HeadCell>
            <Table.HeadCell>Last Seen</Table.HeadCell>
          </Table.Head>
          <Table.Body>
            {workers.map((w) => (
              <Table.Row key={w.id}>
                <Table.Cell>
                  <span class='text-text-primary text-sm font-mono'>{w.name}</span>
                </Table.Cell>
                <Table.Cell>
                  <StatusBadge status={w.status} />
                </Table.Cell>
                <Table.Cell>
                  <span class='text-text-secondary text-sm'>{w.lastSeen}</span>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table>
      </DashboardTableSection>

      <DashboardTableSection
        title='Projects'
        emptyMessage='No projects configured.'
        isEmpty={projects.length === 0}
      >
        <Table>
          <Table.Head>
            <Table.HeadCell>Project</Table.HeadCell>
            <Table.HeadCell>Enabled</Table.HeadCell>
          </Table.Head>
          <Table.Body>
            {projects.map((p) => (
              <Table.Row key={p.id}>
                <Table.Cell>
                  <span class='text-text-primary text-sm font-mono'>
                    {p.name || p.externalProjectId}
                  </span>
                </Table.Cell>
                <Table.Cell>
                  {p.enabled ? (
                    <span class='text-success text-xs uppercase tracking-wider'>Yes</span>
                  ) : (
                    <span class='text-text-muted text-xs uppercase tracking-wider'>No</span>
                  )}
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table>
      </DashboardTableSection>

      <DashboardTableSection
        title='Providers'
        emptyMessage='No providers configured.'
        isEmpty={providers.length === 0}
      >
        <Table>
          <Table.Head>
            <Table.HeadCell>Name</Table.HeadCell>
            <Table.HeadCell>Type</Table.HeadCell>
          </Table.Head>
          <Table.Body>
            {providers.map((p) => (
              <Table.Row key={p.id}>
                <Table.Cell>
                  <span class='text-text-primary text-sm'>{p.name}</span>
                </Table.Cell>
                <Table.Cell>
                  <span class='text-text-secondary text-sm'>{p.providerType}</span>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table>
      </DashboardTableSection>

      <DashboardTableSection
        title='GitHub App'
        emptyMessage='No GitHub App installed.'
        isEmpty={!githubInstallation && !githubLoading}
      >
        <Table>
          <Table.Head>
            <Table.HeadCell>Status</Table.HeadCell>
            <Table.HeadCell>Account</Table.HeadCell>
          </Table.Head>
          <Table.Body>
            <Table.Row>
              <Table.Cell>
                {githubLoading ? (
                  <span class='text-text-muted text-xs animate-pulse'>Loading...</span>
                ) : githubInstallation ? (
                  <span class='text-success text-xs uppercase tracking-wider px-2 py-0.5 border border-success-border bg-success-bg'>
                    Connected
                  </span>
                ) : (
                  <span class='text-text-muted text-xs uppercase tracking-wider px-2 py-0.5 border border-border-base bg-bg-hover'>
                    Not Connected
                  </span>
                )}
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>
                  {githubInstallation ? githubInstallation.accountLogin : '—'}
                </span>
              </Table.Cell>
            </Table.Row>
          </Table.Body>
        </Table>
      </DashboardTableSection>
    </div>
  </div>
);
