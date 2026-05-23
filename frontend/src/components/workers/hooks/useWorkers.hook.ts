import { useSignal, useSignalEffect } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { deleteWorker, generateCode, listWorkers } from '../../../services/workers/workers.service';
import type { Worker } from '../../../types/workers';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

const formatRelativeTime = (dateStr: string | null): string => {
  if (!dateStr) return '—';

  const diff = Date.now() - new Date(dateStr).getTime();
  const seconds = Math.floor(diff / 1000);

  if (seconds < 60) return 'Just now';

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-minutes, 'minute');
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-hours, 'hour');
  }

  const days = Math.floor(hours / 24);
  return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-days, 'day');
};

export interface FormattedWorker {
  id: string;
  name: string;
  status: Worker['status'];
  lastSeen: string;
}

const formatWorkers = (workers: Worker[]): FormattedWorker[] =>
  workers.map((w) => ({
    id: w.id,
    name: w.name,
    status: w.status,
    lastSeen: formatRelativeTime(w.lastSeen)
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

  const formattedWorkers = workers ? formatWorkers(workers) : [];

  const deleteError = useSignal<string | null>(null);
  const deletingId = useSignal<string | null>(null);

  const handleGenerateCode = useCallback(() => {
    generateCodeMutation.mutate(undefined);
  }, [generateCodeMutation]);

  const handleDeleteWorker = useCallback(
    async (id: string) => {
      deleteError.value = null;
      try {
        await deleteWorkerMutation.mutateAsync(id);
      } catch (_err) {
        deleteError.value = 'Failed to delete worker';
      } finally {
        deletingId.value = null;
      }
    },
    [deleteWorkerMutation]
  );

  const handleConfirmDelete = useCallback((id: string) => {
    deletingId.value = id;
  }, []);

  const handleCancelDelete = useCallback(() => {
    deletingId.value = null;
  }, []);

  const codeCountdown = useCodeCountdown(generateCodeMutation.data?.expiresAt ?? null);

  return {
    formattedWorkers,
    code: generateCodeMutation.data?.code ?? null,
    countdown: codeCountdown,
    generateLoading: generateCodeMutation.isPending,
    deletingId,
    deleteError,
    loading,
    error,
    handleGenerateCode,
    handleConfirmDelete,
    handleCancelDelete,
    handleDeleteWorker
  };
};
