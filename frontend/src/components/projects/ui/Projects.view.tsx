import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ProjectConfig } from '../../../types/projects';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
import { Table } from '../../shared/ui/Table.view';
import { WarningBanner } from '../../shared/ui/WarningBanner.view';

interface ProjectsViewProps {
  data: {
    projects: ProjectConfig[];
    deleteConfirmId: Signal<string | null>;
    deleteError: Signal<string | null>;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
  };
  extra: {
    canCreateProject: boolean;
    projectSetupWarning: string;
  };
  actions: {
    onEditClick: (id: string) => void;
    onConnectProject: () => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDelete: (id: string) => void;
  };
}

const columnsTriad = (pickup: string, progress: string, target: string): string =>
  `${pickup} → ${progress} → ${target}`;

export const ProjectsView = ({
  data: { projects, deleteConfirmId, deleteError },
  status: { loading, error },
  extra: { canCreateProject, projectSetupWarning },
  actions: { onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }
}: ProjectsViewProps): JSX.Element => (
  <div class='flex flex-col gap-4'>
    <SectionHeader
      title='Projects'
      hint='Connect projects to a task tracker provider and configure run sync.'
      action={
        <Button
          variant='primary'
          class='shrink-0 whitespace-nowrap px-5'
          onClick={onConnectProject}
          disabled={!canCreateProject}
          title={projectSetupWarning || undefined}
        >
          Connect Project
        </Button>
      }
    />

    {error && <ErrorBanner message={error.message} />}

    {projectSetupWarning && <WarningBanner message={projectSetupWarning} />}

    {deleteError.value && <ErrorBanner message={deleteError.value} />}

    {loading && <div class='text-text-muted text-sm'>Loading projects...</div>}

    {!loading && !error && projects.length === 0 && (
      <EmptyState
        title='No project configs configured yet.'
        description='Add a project config to start monitoring Kaneo projects and creating work runs.'
      />
    )}

    {!loading && projects.length > 0 && (
      <Table>
        <Table.Head>
          <Table.HeadCell>Project</Table.HeadCell>
          <Table.HeadCell>Enabled</Table.HeadCell>
          <Table.HeadCell>Columns</Table.HeadCell>
          <Table.HeadCell>Repo URL</Table.HeadCell>
          <Table.HeadCell>Actions</Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {projects.map((project) => (
            <Table.Row key={project.id}>
              <Table.Cell>
                <span class='text-text-primary text-sm font-mono'>
                  {project.name || project.externalProjectId}
                </span>
              </Table.Cell>
              <Table.Cell>
                {project.enabled ? (
                  <span class='text-success text-xs uppercase tracking-wider'>Yes</span>
                ) : (
                  <span class='text-text-muted text-xs uppercase tracking-wider'>No</span>
                )}
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>
                  {columnsTriad(project.pickupColumn, project.progressColumn, project.targetColumn)}
                </span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono truncate max-w-xs block'>
                  {project.repoUrl || '—'}
                </span>
              </Table.Cell>
              <Table.Cell>
                <ConfirmDelete
                  itemId={project.id}
                  deletingId={deleteConfirmId}
                  onConfirm={onConfirmDelete}
                  onDelete={onDelete}
                  onCancel={onCancelDelete}
                  editActions={
                    <Button variant='ghost' onClick={() => onEditClick(project.id)}>
                      Edit
                    </Button>
                  }
                />
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table>
    )}
  </div>
);
