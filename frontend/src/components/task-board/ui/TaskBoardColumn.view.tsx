import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardColumn as TaskBoardColumnModel } from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';
import type { TaskBoardColumnRole, TaskBoardColumnRoles } from '../types';
import { TaskCard } from './TaskCard.view';

interface TaskBoardColumnProps {
  column: TaskBoardColumnModel;
  visibleCount: number;
  statusOptions: SelectOption[];
  columnRoles: TaskBoardColumnRoles;
  moving: boolean;
  movingTaskId: string | null;
  actionMenuTaskId: string | null;
  onOpenTask: (task: TaskBoardColumnModel['tasks'][number]) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onDragStart: (taskId: string) => void;
  onDragOver: (event: DragEvent) => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onLoadMoreColumn: (columnSlug: string) => void;
  onColumnScroll: (event: Event, columnSlug: string) => void;
}

const ROLE_LABELS: Record<TaskBoardColumnRole, string> = {
  pickup: 'Pickup',
  progress: 'In progress',
  done: 'Done',
  review: 'Review'
};

const ROLE_ORDER: TaskBoardColumnRole[] = ['pickup', 'progress', 'done', 'review'];

const columnRoleActive = (
  role: TaskBoardColumnRole,
  columnSlug: string,
  columnRoles: TaskBoardColumnRoles
): boolean => {
  if (role === 'pickup') return columnRoles.pickupColumn === columnSlug;
  if (role === 'progress') return columnRoles.progressColumn === columnSlug;
  if (role === 'done') return columnRoles.targetColumn === columnSlug;
  return columnRoles.reviewPickupColumn === columnSlug;
};

export const TaskBoardColumn = ({
  column,
  visibleCount,
  statusOptions,
  columnRoles,
  moving,
  movingTaskId,
  actionMenuTaskId,
  onOpenTask,
  onMoveTask,
  onOpenTaskMenu,
  onDragStart,
  onDragOver,
  onDropOnStatus,
  onLoadMoreColumn,
  onColumnScroll
}: TaskBoardColumnProps): JSX.Element => {
  const visibleTasks = column.tasks.slice(0, visibleCount);
  const hasMoreTasks = visibleTasks.length < column.tasks.length;
  const activeRoles = ROLE_ORDER.filter((role) => columnRoleActive(role, column.slug, columnRoles));

  const dropOnStatus = useCallback(
    (event: JSX.TargetedDragEvent<HTMLElement>) => {
      onDropOnStatus(event as unknown as DragEvent, column.slug);
    },
    [column.slug, onDropOnStatus]
  );

  const scrollColumn = useCallback(
    (event: JSX.TargetedEvent<HTMLDivElement>) => {
      onColumnScroll(event, column.slug);
    },
    [column.slug, onColumnScroll]
  );

  const loadMoreColumn = useCallback(() => {
    onLoadMoreColumn(column.slug);
  }, [column.slug, onLoadMoreColumn]);

  return (
    <section
      role='list'
      onDragOver={onDragOver}
      onDrop={dropOnStatus}
      class='flex min-h-80 flex-col gap-4 border border-border-base bg-bg-card p-4'
    >
      <div class='flex flex-col gap-3 border-b border-border-base pb-3'>
        <div class='flex items-start justify-between gap-3'>
          <div class='flex flex-col gap-2'>
            <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
              {column.name}
            </h3>
            <div class='flex flex-wrap gap-1'>
              {activeRoles.length > 0 ? (
                activeRoles.map((role) => (
                  <span
                    key={role}
                    class='border border-accent bg-bg-hover px-2 py-0.5 text-[10px] uppercase tracking-wider text-text-primary'
                  >
                    {ROLE_LABELS[role]}
                  </span>
                ))
              ) : (
                <span class='text-[10px] uppercase tracking-wider text-text-muted'>No role</span>
              )}
            </div>
          </div>
          <span class='text-xs tabular-nums text-text-muted'>{column.tasks.length}</span>
        </div>
      </div>
      <div class='flex max-h-[70vh] flex-col gap-3 overflow-auto pr-1' onScroll={scrollColumn}>
        {visibleTasks.length ? (
          visibleTasks.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              statusOptions={statusOptions}
              moving={moving && movingTaskId === task.id}
              menuOpen={actionMenuTaskId === task.id}
              onMoveTask={onMoveTask}
              onOpenTask={onOpenTask}
              onOpenTaskMenu={onOpenTaskMenu}
              onDragStart={onDragStart}
            />
          ))
        ) : (
          <p class='border border-dashed border-border-base p-4 text-xs text-text-muted'>
            Drop tasks here or create a new one for this column.
          </p>
        )}
        {hasMoreTasks && (
          <Button
            type='button'
            variant='ghost'
            onClick={loadMoreColumn}
            class='border border-border-base'
          >
            Load more
          </Button>
        )}
      </div>
    </section>
  );
};
