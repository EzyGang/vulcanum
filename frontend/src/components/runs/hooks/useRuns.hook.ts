import { useComputed, useSignal } from '@preact/signals';
import type { TargetedMouseEvent } from 'preact';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import { cancelRun } from '../../../services/runs/events.service';
import { bulkDeleteRuns, deleteRun, failRun, listRuns } from '../../../services/runs/runs.service';
import type { WorkRunStatus } from '../../../types/runs';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

const PAGE_SIZE = 20;

export const useRuns = () => {
  const statusFilter = useSignal<WorkRunStatus | undefined>(undefined);
  const page = useSignal(0);
  const selectedIds = useSignal<Set<string>>(new Set());
  const showBulkDeleteDialog = useSignal(false);
  const expandedIds = useSignal<Set<string>>(new Set());

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

  const allSelected = useComputed(() => {
    if (displayRuns.length === 0) {
      return false;
    }
    return displayRuns.every((r) => selectedIds.value.has(r.id));
  });

  const someSelected = useComputed(() => {
    if (displayRuns.length === 0) {
      return false;
    }
    return displayRuns.some((r) => selectedIds.value.has(r.id)) && !allSelected.value;
  });

  const selectionCount = useComputed(() => selectedIds.value.size);

  const deleteRunMutation = useApiMutation((id: string) => deleteRun(id), {
    onSuccess: () => invalidate('runs')
  });

  const bulkDeleteMutation = useApiMutation((ids: string[]) => bulkDeleteRuns(ids), {
    onSuccess: () => {
      invalidate('runs');
      selectedIds.value = new Set();
      showBulkDeleteDialog.value = false;
    }
  });

  const failRunMutation = useApiMutation((id: string) => failRun(id), {
    onSuccess: () => invalidate('runs')
  });

  const cancelRunMutation = useApiMutation((id: string) => cancelRun(id).then(() => id), {
    onSuccess: () => {
      invalidate('runs');
      invalidate('run-events');
    }
  });

  const handleCancelRun = useCallback(
    (id: string) => {
      cancelRunMutation.mutate(id);
    },
    [cancelRunMutation]
  );

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
    selectedIds.value = new Set();
  }, []);

  const nextPage = useCallback(() => {
    if (hasNextPage) {
      page.value += 1;
      selectedIds.value = new Set();
    }
  }, [hasNextPage]);

  const prevPage = useCallback(() => {
    if (hasPrevPage) {
      page.value -= 1;
      selectedIds.value = new Set();
    }
  }, [hasPrevPage]);

  const handleToggleSelect = useCallback((id: string) => {
    const next = new Set(selectedIds.value);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedIds.value = next;
  }, []);

  const handleToggleSelectAll = useCallback(() => {
    if (allSelected.value) {
      selectedIds.value = new Set();
    } else {
      selectedIds.value = new Set(displayRuns.map((r) => r.id));
    }
  }, [displayRuns]);

  const handleOpenBulkDelete = useCallback(() => {
    showBulkDeleteDialog.value = true;
  }, []);

  const handleConfirmBulkDelete = useCallback(() => {
    bulkDeleteMutation.mutate(Array.from(selectedIds.value));
  }, [bulkDeleteMutation]);

  const handleCancelBulkDelete = useCallback(() => {
    showBulkDeleteDialog.value = false;
  }, []);

  const handleFailRun = useCallback(
    (id: string) => {
      failRunMutation.mutate(id);
    },
    [failRunMutation]
  );

  const handleToggleExpanded = useCallback((id: string) => {
    const next = new Set(expandedIds.value);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedIds.value = next;
  }, []);

  const handleStopRowToggle = useCallback((event: TargetedMouseEvent<HTMLElement>) => {
    event.stopPropagation();
  }, []);

  const handleToggleExpandedControl = useCallback(
    (id: string, event: TargetedMouseEvent<HTMLElement>) => {
      event.stopPropagation();
      handleToggleExpanded(id);
    },
    [handleToggleExpanded]
  );

  return {
    data: {
      runs: displayRuns,
      selectedIds,
      allSelected: allSelected.value,
      someSelected: someSelected.value,
      selectionCount: selectionCount.value,
      showBulkDeleteDialog,
      expandedIds
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
      handleCancelDelete,
      handleToggleSelect,
      handleToggleSelectAll,
      handleOpenBulkDelete,
      handleConfirmBulkDelete,
      handleCancelBulkDelete,
      handleFailRun,
      handleToggleExpanded,
      handleStopRowToggle,
      handleToggleExpandedControl,
      handleCancelRun
    }
  };
};
