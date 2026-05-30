import { useSignal, useSignalEffect } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import {
  deleteWorker,
  generateCode,
  listWorkers,
  updateWorkerStatus
} from '../../../services/workers/workers.service';
import type { UpdateWorkerStatusRequest, Worker } from '../../../types/workers';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { formatRelativeTime } from '../../../utils/format';

export interface FormattedWorker {
  id: string;
  name: string;
  status: Worker['status'];
  lastSeen: string;
  activeJobs: number;
  maxConcurrentJobs: number;
  consecutiveErrors: number;
}

const formatWorkers = (workers: Worker[]): FormattedWorker[] =>
  workers.map((w) => ({
    id: w.id,
    name: w.name,
    status: w.status,
    lastSeen: formatRelativeTime(w.lastSeen),
    activeJobs: w.activeJobs,
    maxConcurrentJobs: w.maxConcurrentJobs,
    consecutiveErrors: w.consecutiveErrors
  }));

export const useCodeCountdown = (expiresAt: string | null) => {
  const remaining = useSignal('');

  useSignalEffect(() => {
    if (!expiresAt) {
      remaining.value = '';
      return;
    }

    const tick = () => {
      const left = new Date(expiresAt).getTime() - Date.now();
      if (left <= 0) {
        remaining.value = '';
        return;
      }
      const m = Math.floor(left / 60000);
      const s = Math.floor((left % 60000) / 1000);
      remaining.value = `${m}m ${s}s remaining`;
    };

    tick();
    const interval = setInterval(tick, 1000);
    return () => clearInterval(interval);
  });

  return remaining;
};

export const useWorkers = () => {
  const {
    data: workers,
    isLoading: loading,
    error
  } = useApiQuery(['workers'], () => listWorkers());

  const generateCodeMutation = useApiMutation(() => generateCode(), {
    onSuccess: () => invalidate('workers')
  });

  const deleteWorkerMutation = useApiMutation((id: string) => deleteWorker(id), {
    onSuccess: () => invalidate('workers')
  });

  const updateStatusMutation = useApiMutation(
    ({ id, data }: { id: string; data: UpdateWorkerStatusRequest }) => updateWorkerStatus(id, data),
    {
      onSuccess: () => invalidate('workers')
    }
  );

  const formattedWorkers = workers ? formatWorkers(workers) : [];

  const {
    deletingId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete: handleDeleteWorker
  } = useDeleteConfirm('worker', deleteWorkerMutation);

  const handleGenerateCode = useCallback(() => {
    generateCodeMutation.mutate(undefined);
  }, [generateCodeMutation]);

  const handleUpdateStatus = useCallback(
    (id: string, status: UpdateWorkerStatusRequest['status']) => {
      updateStatusMutation.mutate({ id, data: { status } });
    },
    [updateStatusMutation]
  );

  const codeCountdown = useCodeCountdown(generateCodeMutation.data?.expiresAt ?? null);

  return {
    formattedWorkers,
    code: generateCodeMutation.data?.code ?? null,
    countdown: codeCountdown,
    generateLoading: generateCodeMutation.isPending,
    deletingId,
    deleteError,
    updateStatusError: updateStatusMutation.error,
    loading,
    error,
    handleGenerateCode,
    handleConfirmDelete,
    handleCancelDelete,
    handleDeleteWorker,
    handleUpdateStatus
  };
};
