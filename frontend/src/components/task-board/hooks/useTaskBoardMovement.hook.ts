import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import {
  addTaskLabel,
  moveTask,
  removeTaskLabel,
  updateTask
} from '../../../services/task-board/task-board.service';
import type { TaskBoardColumn, TaskBoardLabel, TaskBoardTask } from '../../../types/task-board';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';
import { textInputHandler } from '../../../utils/signal-input';
import { boardQueryKey, COLUMN_PAGE_SIZE } from './taskBoard.helpers';

interface TaskBoardSelection {
  providerId: string;
  externalProjectId: string;
}

export const useTaskBoardMovement = (
  selection: TaskBoardSelection | null,
  columns: TaskBoardColumn[],
  labels: TaskBoardLabel[]
) => {
  const selectedTask = useSignal<TaskBoardTask | null>(null);
  const editTitle = useSignal('');
  const editBody = useSignal('');
  const editLabelIds = useSignal<string[]>([]);
  const editError = useSignal<string | null>(null);
  const draggedTask = useSignal<{ id: string; status: string } | null>(null);
  const dragTargetStatus = useSignal<string | null>(null);
  const actionMenuTaskId = useSignal<string | null>(null);
  const visibleTaskCounts = useSignal<Record<string, number>>({});

  useEffect(() => {
    const nextCounts: Record<string, number> = {};

    for (const column of columns) {
      nextCounts[column.slug] = visibleTaskCounts.value[column.slug] ?? COLUMN_PAGE_SIZE;
    }

    visibleTaskCounts.value = nextCounts;
  }, [columns]);

  useEffect(() => {
    if (actionMenuTaskId.value === null) return;

    const closeOpenMenu = () => {
      actionMenuTaskId.value = null;
    };

    window.addEventListener('click', closeOpenMenu);

    return () => {
      window.removeEventListener('click', closeOpenMenu);
    };
  }, [actionMenuTaskId, actionMenuTaskId.value]);

  const moveMutation = useApiMutation(
    ({ taskId, nextStatus }: { taskId: string; nextStatus: string }) =>
      moveTask(selection?.providerId ?? '', { taskId, status: nextStatus }),
    {
      onSuccess: () => {
        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ taskId, title, body }: { taskId: string; title: string; body: string }) =>
      updateTask(selection?.providerId ?? '', taskId, { title, body }),
    {
      onSuccess: (response) => {
        const labels = selectedTask.value?.labels ?? response.task.labels;
        selectedTask.value = { ...response.task, labels };
        editTitle.value = response.task.title;
        editBody.value = response.task.description ?? '';
        editError.value = null;
        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const labelMutation = useApiMutation(
    ({ taskId, labelId, checked }: { taskId: string; labelId: string; checked: boolean }) =>
      checked
        ? addTaskLabel(selection?.providerId ?? '', taskId, labelId)
        : removeTaskLabel(selection?.providerId ?? '', taskId, labelId),
    {
      onSuccess: (_response, input) => {
        const nextLabelIds = input.checked
          ? [...editLabelIds.value, input.labelId]
          : editLabelIds.value.filter((labelId) => labelId !== input.labelId);
        const nextLabelIdSet = new Set(nextLabelIds);
        editLabelIds.value = nextLabelIds;

        if (selectedTask.value) {
          selectedTask.value = {
            ...selectedTask.value,
            labels: labels.filter((label) => nextLabelIdSet.has(label.id))
          };
        }

        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const moveTaskToStatus = useCallback(
    (taskId: string, nextStatus: string) => {
      actionMenuTaskId.value = null;
      moveMutation.mutate({ taskId, nextStatus });
    },
    [actionMenuTaskId, moveMutation]
  );

  const openTask = useCallback(
    (task: TaskBoardTask) => {
      actionMenuTaskId.value = null;
      selectedTask.value = task;
      editTitle.value = task.title;
      editBody.value = task.description ?? '';
      editLabelIds.value = task.labels.map((label) => label.id);
      editError.value = null;
    },
    [actionMenuTaskId, editBody, editError, editLabelIds, editTitle, selectedTask]
  );

  const closeTask = useCallback(() => {
    selectedTask.value = null;
    editError.value = null;
  }, [editError, selectedTask]);

  const submitTaskEdit = useCallback(
    (event: Event) => {
      event.preventDefault();
      const task = selectedTask.value;
      const title = editTitle.value.trim();

      if (task === null) return;
      if (!title) {
        editError.value = 'Task title is required';
        return;
      }

      updateMutation.mutate({ taskId: task.id, title, body: editBody.value });
    },
    [editBody, editError, editTitle, selectedTask, updateMutation]
  );

  const toggleTaskLabel = useCallback(
    (labelId: string, checked: boolean) => {
      const task = selectedTask.value;
      if (task === null) return;

      labelMutation.mutate({ taskId: task.id, labelId, checked });
    },
    [labelMutation, selectedTask]
  );

  const startDrag = useCallback(
    (taskId: string, taskStatus: string) => {
      draggedTask.value = { id: taskId, status: taskStatus };
      dragTargetStatus.value = null;
    },
    [dragTargetStatus, draggedTask]
  );

  const clearDrag = useCallback(() => {
    draggedTask.value = null;
    dragTargetStatus.value = null;
  }, [dragTargetStatus, draggedTask]);

  const allowDropOnStatus = useCallback(
    (event: DragEvent, nextStatus: string) => {
      event.preventDefault();
      dragTargetStatus.value =
        draggedTask.value !== null && draggedTask.value.status !== nextStatus ? nextStatus : null;
    },
    [dragTargetStatus, draggedTask]
  );

  const dropOnStatus = useCallback(
    (event: DragEvent, nextStatus: string) => {
      event.preventDefault();
      const task = draggedTask.value;
      clearDrag();
      if (task === null || task.status === nextStatus) return;

      moveMutation.mutate({ taskId: task.id, nextStatus });
    },
    [clearDrag, draggedTask, moveMutation]
  );

  const openTaskMenu = useCallback(
    (event: MouseEvent, taskId: string) => {
      event.preventDefault();
      event.stopPropagation();
      actionMenuTaskId.value = actionMenuTaskId.value === taskId ? null : taskId;
    },
    [actionMenuTaskId]
  );

  const closeTaskMenu = useCallback(() => {
    actionMenuTaskId.value = null;
  }, [actionMenuTaskId]);

  const loadMoreColumn = useCallback(
    (columnSlug: string) => {
      visibleTaskCounts.value = {
        ...visibleTaskCounts.value,
        [columnSlug]: (visibleTaskCounts.value[columnSlug] ?? COLUMN_PAGE_SIZE) + COLUMN_PAGE_SIZE
      };
    },
    [visibleTaskCounts]
  );

  const scrollColumn = useCallback(
    (event: Event, columnSlug: string) => {
      const target = event.currentTarget as HTMLElement;
      const nearBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 32;

      if (nearBottom) {
        loadMoreColumn(columnSlug);
      }
    },
    [loadMoreColumn]
  );

  return {
    data: {
      selectedTask: selectedTask.value,
      actionMenuTaskId: actionMenuTaskId.value,
      visibleTaskCounts: visibleTaskCounts.value,
      dropPreviewColumn: dragTargetStatus.value
    },
    form: {
      editTitle: editTitle.value,
      editBody: editBody.value,
      editLabelIds: editLabelIds.value,
      editError: editError.value
    },
    status: {
      movingTaskId: moveMutation.variables?.taskId ?? null,
      moving: moveMutation.isPending,
      updatingTask: updateMutation.isPending,
      updatingTaskLabel: labelMutation.isPending
    },
    error:
      moveMutation.error?.message ??
      updateMutation.error?.message ??
      labelMutation.error?.message ??
      null,
    actions: {
      onMoveTask: moveTaskToStatus,
      onOpenTask: openTask,
      onCloseTask: closeTask,
      onTaskDetailsOpenChange: (open: boolean) => {
        if (!open) closeTask();
      },
      onEditTaskTitleInput: textInputHandler(editTitle),
      onEditTaskBodyInput: textInputHandler(editBody),
      onSubmitTaskEdit: submitTaskEdit,
      onToggleTaskLabel: toggleTaskLabel,
      onDragStart: startDrag,
      onDragOverStatus: allowDropOnStatus,
      onDragEnd: clearDrag,
      onDropOnStatus: dropOnStatus,
      onOpenTaskMenu: openTaskMenu,
      onCloseTaskMenu: closeTaskMenu,
      onLoadMoreColumn: loadMoreColumn,
      onColumnScroll: scrollColumn
    }
  };
};
