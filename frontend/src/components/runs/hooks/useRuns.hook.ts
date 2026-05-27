import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { deleteRun, listRuns } from '../../../services/runs/runs.service';
import type { WorkRunStatus } from '../../../types/runs';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

const PAGE_SIZE = 20;

export const useRuns = () => {
  const statusFilter = useSignal<WorkRunStatus | undefined>(undefined);
  const page = useSignal(0);

  const {
    data: runs,
    isLoading: loading,
    error
  } = useApiQuery(['runs', { status: statusFilter.value, page: page.value }], () =>
    listRuns({
      status: statusFilter.value,
      limit: PAGE_SIZE + 1,
      offset: page.value * PAGE_SIZE
    })
  );

  const hasNextPage = runs ? runs.length > PAGE_SIZE : false;
  const displayRuns = runs ? runs.slice(0, PAGE_SIZE) : [];
  const hasPrevPage = page.value > 0;

  const deleteRunMutation = useApiMutation((id: string) => deleteRun(id), {
    onSuccess: () => invalidate(['runs'])
  });

  const deleteError = useSignal<string | null>(null);
  const deletingId = useSignal<string | null>(null);

  const setStatusFilter = useCallback((status: WorkRunStatus | undefined) => {
    statusFilter.value = status;
    page.value = 0;
  }, []);

  const nextPage = useCallback(() => {
    if (hasNextPage) {
      page.value += 1;
    }
  }, [hasNextPage]);

  const prevPage = useCallback(() => {
    if (hasPrevPage) {
      page.value -= 1;
    }
  }, [hasPrevPage]);

  const handleDeleteRun = useCallback(
    async (id: string) => {
      deleteError.value = null;
      try {
        await deleteRunMutation.mutateAsync(id);
      } catch (_err) {
        deleteError.value = 'Failed to delete run';
      } finally {
        deletingId.value = null;
      }
    },
    [deleteRunMutation]
  );

  const handleConfirmDelete = useCallback((id: string) => {
    deletingId.value = id;
  }, []);

  const handleCancelDelete = useCallback(() => {
    deletingId.value = null;
  }, []);

  return {
    data: {
      runs: displayRuns
    },
    status: {
      loading,
      error,
      deleteError,
      deletingId,
      statusFilter,
      page,
      hasNextPage,
      hasPrevPage
    },
    actions: {
      setStatusFilter,
      nextPage,
      prevPage,
      handleDeleteRun,
      handleConfirmDelete,
      handleCancelDelete
    }
  };
};
