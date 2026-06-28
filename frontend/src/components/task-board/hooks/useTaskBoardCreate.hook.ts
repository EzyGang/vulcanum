import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { createTask } from '../../../services/task-board/task-board.service';
import type { TaskBoardColumn } from '../../../types/task-board';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';
import { textInputHandler } from '../../../utils/signal-input';
import { boardQueryKey, firstColumnSlug } from './taskBoard.helpers';

interface TaskBoardSelection {
  providerId: string;
  externalProjectId: string;
}

export const useTaskBoardCreate = (
  selection: TaskBoardSelection | null,
  columns: TaskBoardColumn[]
) => {
  const title = useSignal('');
  const body = useSignal('');
  const status = useSignal('');
  const createError = useSignal<string | null>(null);
  const createDialogOpen = useSignal(false);

  useEffect(() => {
    if (!columns.length) {
      status.value = '';
      return;
    }

    const statusStillExists = columns.some((column) => column.slug === status.value);
    if (!statusStillExists) {
      status.value = firstColumnSlug(columns);
    }
  }, [columns, status.value]);

  const createMutation = useApiMutation(
    () =>
      createTask(selection?.providerId ?? '', selection?.externalProjectId ?? '', {
        title: title.value.trim(),
        body: body.value,
        status: status.value || undefined
      }),
    {
      onSuccess: () => {
        title.value = '';
        body.value = '';
        createError.value = null;
        createDialogOpen.value = false;
        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const submitTask = useCallback(
    (event: Event) => {
      event.preventDefault();
      if (!title.value.trim()) {
        createError.value = 'Task title is required';
        return;
      }

      createMutation.mutate(undefined);
    },
    [createMutation, title, createError]
  );

  const selectStatus = useCallback(
    (nextStatus: string) => {
      status.value = nextStatus;
    },
    [status]
  );

  return {
    form: {
      title: title.value,
      body: body.value,
      status: status.value,
      createError: createError.value
    },
    status: {
      creating: createMutation.isPending
    },
    error: createMutation.error?.message ?? null,
    dialogOpen: createDialogOpen.value,
    actions: {
      onTitleInput: textInputHandler(title),
      onBodyInput: textInputHandler(body),
      onStatusChange: selectStatus,
      onSubmitTask: submitTask,
      onOpenCreateTask: () => {
        createDialogOpen.value = true;
      },
      onCloseCreateTask: () => {
        createDialogOpen.value = false;
      },
      onCreateDialogOpenChange: (open: boolean) => {
        if (!open) createDialogOpen.value = false;
      }
    }
  };
};
