import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import { deleteProject, listProjects } from '../../../services/projects/projects.service';
import { listProviders } from '../../../services/providers/providers.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useProjects = () => {
  const {
    data: projects,
    isLoading: loading,
    error
  } = useApiQuery(['projects'], () => listProjects());

  const { data: providers = [] } = useApiQuery(['providers'], () => listProviders());

  const deleteMutation = useApiMutation((id: string) => deleteProject(id), {
    onSuccess: () => invalidate('projects')
  });

  const {
    deletingId: deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  } = useDeleteConfirm('project config', deleteMutation);

  return {
    projects: projects ?? [],
    providers,
    loading,
    error,
    deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  };
};
