import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { ProjectConfig } from '../../../types/projects';
import type { ApiError } from '../../../utils/api/client';

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
  actions: {
    onEditClick: (id: string) => void;
    onNewProject: () => void;
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
  actions: { onEditClick, onNewProject, onConfirmDelete, onCancelDelete, onDelete }
}: ProjectsViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <section class='flex flex-col gap-4'>
      <div class='flex items-center justify-between'>
        <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
          Project Configs
        </h2>
        <button
          type='button'
          onClick={onNewProject}
          class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider
                 px-4 py-3 hover:opacity-90 transition-opacity'
        >
          New Project
        </button>
      </div>

      {error && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {error.message}
        </div>
      )}

      {deleteError.value && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {deleteError.value}
        </div>
      )}

      {loading && <div class='text-text-muted text-sm'>Loading projects...</div>}

      {!loading && !error && projects.length === 0 && (
        <div class='flex flex-col items-center gap-4 bg-bg-card border border-border-base p-12'>
          <p class='text-text-muted text-sm'>No project configs configured yet.</p>
          <p class='text-text-muted text-xs'>
            Add a project config to start monitoring Kaneo projects and creating work runs.
          </p>
        </div>
      )}

      {!loading && projects.length > 0 && (
        <div class='overflow-x-auto'>
          <table class='w-full border-collapse'>
            <thead>
              <tr class='border-b border-border-base'>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Kaneo Project ID
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Enabled
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Columns
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Repo URL
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {projects.map((project) => (
                <tr key={project.id} class='border-b border-border-base'>
                  <td class='px-5 py-3'>
                    <span class='text-text-primary text-sm font-mono'>
                      {project.kaneoProjectId}
                    </span>
                  </td>
                  <td class='px-5 py-3'>
                    {project.enabled ? (
                      <span class='text-success text-xs uppercase tracking-wider'>Yes</span>
                    ) : (
                      <span class='text-text-muted text-xs uppercase tracking-wider'>No</span>
                    )}
                  </td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm font-mono'>
                      {columnsTriad(
                        project.pickupColumn,
                        project.progressColumn,
                        project.targetColumn
                      )}
                    </span>
                  </td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm font-mono truncate max-w-xs block'>
                      {project.repoUrl || '—'}
                    </span>
                  </td>
                  <td class='px-5 py-3'>
                    {deleteConfirmId.value === project.id ? (
                      <div class='flex items-center gap-2'>
                        <span class='text-text-muted text-xs'>Confirm?</span>
                        <button
                          type='button'
                          onClick={() => onDelete(project.id)}
                          class='text-error text-xs uppercase tracking-wider hover:opacity-80 transition-opacity'
                        >
                          Delete
                        </button>
                        <button
                          type='button'
                          onClick={onCancelDelete}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors'
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <div class='flex items-center gap-3'>
                        <button
                          type='button'
                          onClick={() => onEditClick(project.id)}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors'
                        >
                          Edit
                        </button>
                        <button
                          type='button'
                          onClick={() => onConfirmDelete(project.id)}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-error transition-colors'
                        >
                          Delete
                        </button>
                      </div>
                    )}
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
