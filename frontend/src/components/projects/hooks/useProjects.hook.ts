import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { deleteProject, listProjects } from '../../../services/projects/projects.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useProjects = () => {
  const {
    data: projects,
    isLoading: loading,
    error
  } = useApiQuery(['projects'], () => listProjects());

  const deleteMutation = useApiMutation((id: string) => deleteProject(id), {
    onSuccess: () => invalidate('projects')
  });

  const deleteError = useSignal<string | null>(null);
  const deleteConfirmId = useSignal<string | null>(null);

  const handleDelete = useCallback(
    async (id: string) => {
      deleteError.value = null;
      try {
        await deleteMutation.mutateAsync(id);
      } catch (_err) {
        deleteError.value = 'Failed to delete project config';
      } finally {
        deleteConfirmId.value = null;
      }
    },
    [deleteMutation]
  );

  const handleConfirmDelete = useCallback((id: string) => {
    deleteConfirmId.value = id;
  }, []);

  const handleCancelDelete = useCallback(() => {
    deleteConfirmId.value = null;
  }, []);

  return {
    projects: projects ?? [],
    loading,
    error,
    deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  };
};
