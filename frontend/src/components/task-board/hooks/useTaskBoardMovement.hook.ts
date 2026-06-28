import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { moveTask } from '../../../services/task-board/task-board.service';
import type { TaskBoardColumn, TaskBoardTask } from '../../../types/task-board';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';
import { boardQueryKey, COLUMN_PAGE_SIZE } from './taskBoard.helpers';

interface TaskBoardSelection {
  providerId: string;
  externalProjectId: string;
}

export const useTaskBoardMovement = (
  selection: TaskBoardSelection | null,
  columns: TaskBoardColumn[]
) => {
  const selectedTask = useSignal<TaskBoardTask | null>(null);
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
    },
    [actionMenuTaskId, selectedTask]
  );

  const closeTask = useCallback(() => {
    selectedTask.value = null;
  }, [selectedTask]);

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
    status: {
      movingTaskId: moveMutation.variables?.taskId ?? null,
      moving: moveMutation.isPending
    },
    error: moveMutation.error?.message ?? null,
    actions: {
      onMoveTask: moveTaskToStatus,
      onOpenTask: openTask,
      onCloseTask: closeTask,
      onTaskDetailsOpenChange: (open: boolean) => {
        if (!open) closeTask();
      },
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
