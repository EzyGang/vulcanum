import type { JSX } from 'preact';
import { useLocation } from 'wouter-preact';
import { useProjects } from '../hooks/useProjects.hook';
import { ProjectsView } from '../ui/Projects.view';

export const ProjectsContainer = (): JSX.Element => {
  const [_, setLocation] = useLocation();
  const {
    projects,
    loading,
    error,
    deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  } = useProjects();

  return (
    <ProjectsView
      data={{ projects, deleteConfirmId, deleteError }}
      status={{ loading, error }}
      actions={{
        onEditClick: (id: string) => setLocation(`/projects/${id}/edit`),
        onConnectProject: () => setLocation('/projects/connect'),
        onConfirmDelete: handleConfirmDelete,
        onCancelDelete: handleCancelDelete,
        onDelete: handleDelete
      }}
    />
  );
};
