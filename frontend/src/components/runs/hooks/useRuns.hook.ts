import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
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
    onSuccess: () => invalidate('runs')
  });

  const {
    deletingId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete: handleDeleteRun
  } = useDeleteConfirm('run', deleteRunMutation);

  const setStatusFilter = useCallback((value: string) => {
    statusFilter.value = value ? (value as WorkRunStatus) : undefined;
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
