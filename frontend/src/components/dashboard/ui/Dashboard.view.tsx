import type { JSX } from 'preact';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { Card } from '../../shared/ui/Card.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { Table } from '../../shared/ui/Table.view';

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
    goToProjectSettings: () => void;
  };
}

const StatCard = ({ label, value }: { label: string; value: number }): JSX.Element => (
  <Card class='flex flex-col gap-1'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <span class='text-text-primary text-2xl font-semibold font-mono'>{value}</span>
  </Card>
);

const SkeletonStatCard = ({ label }: { label: string }): JSX.Element => (
  <Card class='flex flex-col gap-2'>
    <span class='text-text-muted text-xs uppercase tracking-wider'>{label}</span>
    <div class='h-8 w-12 bg-bg-hover animate-pulse' />
  </Card>
);

const SectionHeader = ({
  title,
  onViewAll
}: {
  title: string;
  onViewAll: () => void;
}): JSX.Element => (
  <div class='flex items-center justify-between'>
    <h3 class='text-md font-semibold text-text-primary uppercase tracking-wide'>{title}</h3>
    <Button variant='ghost' onClick={onViewAll}>
      View all →
    </Button>
  </div>
);

export const DashboardView = ({
  data: { stats, workers, projects, providers, githubInstallation, githubLoading },
  status: { loading, error },
  actions: { goToSettings, goToWorkers, goToRuns, goToNewProject, goToProjectSettings }
}: DashboardViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
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

    <Card class='flex flex-col gap-4 animate-slide-up'>
      <h3 class='text-md font-semibold text-text-primary uppercase tracking-wide'>Quick Actions</h3>
      <div class='flex flex-wrap gap-3'>
        <Button variant='secondary' onClick={goToSettings}>
          Add Provider
        </Button>
        <Button variant='secondary' onClick={goToNewProject}>
          Connect Project
        </Button>
        <Button variant='secondary' onClick={goToRuns}>
          View Runs
        </Button>
        <Button variant='secondary' onClick={goToWorkers}>
          Manage Workers
        </Button>
      </div>
    </Card>

    <section class='flex flex-col gap-4 animate-slide-up' style='animation-delay: 50ms'>
      <SectionHeader title='Workers' onViewAll={goToWorkers} />
      {workers.length === 0 ? (
        <p class='text-text-muted text-sm'>No workers registered.</p>
      ) : (
        <div class='max-h-56 overflow-y-auto'>
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
        </div>
      )}
    </section>

    <section class='flex flex-col gap-4 animate-slide-up' style='animation-delay: 100ms'>
      <SectionHeader title='Projects' onViewAll={goToProjectSettings} />
      {projects.length === 0 ? (
        <p class='text-text-muted text-sm'>No projects configured.</p>
      ) : (
        <div class='max-h-56 overflow-y-auto'>
          <Table>
            <Table.Head>
              <Table.HeadCell>Project ID</Table.HeadCell>
              <Table.HeadCell>Enabled</Table.HeadCell>
            </Table.Head>
            <Table.Body>
              {projects.map((p) => (
                <Table.Row key={p.id}>
                  <Table.Cell>
                    <span class='text-text-primary text-sm font-mono'>{p.externalProjectId}</span>
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
        </div>
      )}
    </section>

    <section class='flex flex-col gap-4 animate-slide-up' style='animation-delay: 150ms'>
      <SectionHeader title='Providers' onViewAll={goToSettings} />
      {providers.length === 0 ? (
        <p class='text-text-muted text-sm'>No providers configured.</p>
      ) : (
        <div class='max-h-56 overflow-y-auto'>
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
        </div>
      )}
    </section>

    <section class='flex flex-col gap-4 animate-slide-up' style='animation-delay: 200ms'>
      <SectionHeader title='GitHub App' onViewAll={goToSettings} />
      <Card class='flex items-center justify-between'>
        <div class='flex items-center gap-3'>
          <span class='text-text-primary text-sm font-semibold uppercase tracking-wider'>
            Status
          </span>
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
        </div>
        {githubInstallation && (
          <span class='text-text-secondary text-sm font-mono'>
            {githubInstallation.accountLogin}
          </span>
        )}
      </Card>
    </section>
  </div>
);
