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

export type WorkerRegistrationCopyTarget = 'code' | 'setup-command';

const API_SUFFIX = /\/api\/v1\/?$/;

const trimTrailingSlash = (value: string): string => value.replace(/\/+$/, '');

const resolveInstanceUrl = (): string => {
  const apiUrl = import.meta.env.VITE_API_URL;
  if (apiUrl) {
    return trimTrailingSlash(apiUrl.replace(API_SUFFIX, ''));
  }

  if (typeof window === 'undefined') {
    return '<instance>';
  }

  return window.location.origin;
};

const maskRegistrationCode = (code: string | null): string | null => {
  if (!code) {
    return null;
  }

  const suffix = code.slice(-4);
  return `•••• •••• •••• ${suffix}`;
};

const buildSetupCommand = (instanceUrl: string, code: string): string =>
  `vulcanum worker setup --instance ${instanceUrl} --code ${code}`;

const copyToClipboard = async (text: string): Promise<void> => {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return;
    } catch {
      // Fall back to selection copy below for browsers that expose Clipboard API but deny it.
    }
  }

  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.top = '0';
  textarea.style.opacity = '0';
  document.body.append(textarea);
  textarea.focus();
  textarea.select();
  const copied = document.execCommand('copy');
  textarea.remove();

  if (!copied) {
    throw new Error('Clipboard copy failed');
  }
};

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
  const copiedTarget = useSignal<WorkerRegistrationCopyTarget | null>(null);
  const copyError = useSignal<string | null>(null);
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
    copyError.value = null;
    copiedTarget.value = null;
    generateCodeMutation.mutate(undefined);
  }, [copyError, copiedTarget, generateCodeMutation]);

  const handleUpdateStatus = useCallback(
    (id: string, status: UpdateWorkerStatusRequest['status']) => {
      updateStatusMutation.mutate({ id, data: { status } });
    },
    [updateStatusMutation]
  );

  useSignalEffect(() => {
    if (!copiedTarget.value) {
      return;
    }

    const timeout = window.setTimeout(() => {
      copiedTarget.value = null;
    }, 1800);

    return () => window.clearTimeout(timeout);
  });

  const generatedCode = generateCodeMutation.data?.code ?? null;
  const instanceUrl = resolveInstanceUrl();
  const setupCommand = generatedCode ? buildSetupCommand(instanceUrl, generatedCode) : null;
  const codeCountdown = useCodeCountdown(generateCodeMutation.data?.expiresAt ?? null);

  const copyGeneratedCode = useCallback(async () => {
    if (!generatedCode) {
      return;
    }

    try {
      await copyToClipboard(generatedCode);
      copyError.value = null;
      copiedTarget.value = 'code';
    } catch {
      copyError.value = 'Copy failed. Check browser clipboard permissions and try again.';
    }
  }, [copiedTarget, copyError, generatedCode]);

  const copySetupCommand = useCallback(async () => {
    if (!setupCommand) {
      return;
    }

    try {
      await copyToClipboard(setupCommand);
      copyError.value = null;
      copiedTarget.value = 'setup-command';
    } catch {
      copyError.value = 'Copy failed. Check browser clipboard permissions and try again.';
    }
  }, [copiedTarget, copyError, setupCommand]);

  return {
    formattedWorkers,
    maskedCode: maskRegistrationCode(generatedCode),
    setupCommandPreview: generatedCode
      ? buildSetupCommand(instanceUrl, maskRegistrationCode(generatedCode) ?? '')
      : null,
    countdown: codeCountdown,
    generateLoading: generateCodeMutation.isPending,
    copiedTarget,
    copyError,
    deletingId,
    deleteError,
    updateStatusError: updateStatusMutation.error,
    loading,
    error,
    handleGenerateCode,
    handleConfirmDelete,
    handleCancelDelete,
    handleDeleteWorker,
    handleUpdateStatus,
    copyGeneratedCode,
    copySetupCommand
  };
};
