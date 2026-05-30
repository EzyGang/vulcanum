import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';

export const useDeleteConfirm = (
  entityName: string,
  deleteMutation: { mutateAsync: (id: string) => Promise<unknown> }
) => {
  const deletingId = useSignal<string | null>(null);
  const deleteError = useSignal<string | null>(null);

  const handleConfirmDelete = useCallback((id: string) => {
    deletingId.value = id;
  }, []);

  const handleCancelDelete = useCallback(() => {
    deletingId.value = null;
  }, []);

  const handleDelete = useCallback(
    async (id: string) => {
      deleteError.value = null;
      try {
        await deleteMutation.mutateAsync(id);
      } catch {
        deleteError.value = `Failed to delete ${entityName}`;
      } finally {
        deletingId.value = null;
      }
    },
    [deleteMutation, entityName]
  );

  return { deletingId, deleteError, handleConfirmDelete, handleCancelDelete, handleDelete };
};
