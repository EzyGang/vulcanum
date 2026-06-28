import { useSignal } from '@preact/signals';
import { IconSettings } from '@tabler/icons-react';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import { useCallback, useEffect } from 'preact/hooks';
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
  configuringColumns: boolean;
  dropPreviewColumn: string | null;
  onOpenTask: (task: TaskBoardColumnModel['tasks'][number]) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onDragStart: (taskId: string, status: string) => void;
  onDragOverStatus: (event: DragEvent, status: string) => void;
  onDragEnd: () => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
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

const ROLE_HELP: Record<TaskBoardColumnRole, string> = {
  pickup: 'Ready work. Agents and humans can pick tickets up from this column.',
  progress: 'Active work. Moving tickets here marks them as being worked on.',
  done: 'Completed work. Moving tickets here closes the board workflow.',
  review: 'Review pickup. Automated review work starts from this column when enabled.'
};

const ROLE_BADGE_CLASSES: Record<TaskBoardColumnRole, string> = {
  pickup: 'border-warning-border bg-warning-bg text-warning',
  progress: 'border-accent/60 bg-accent/10 text-accent-light',
  done: 'border-success-border bg-success-bg text-success',
  review: 'border-border-focus bg-bg-active text-text-primary'
};

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

interface RoleBadgeProps {
  role: TaskBoardColumnRole;
}

const RoleBadge = ({ role }: RoleBadgeProps): JSX.Element => (
  <span
    title={ROLE_HELP[role]}
    class={clsx(
      'select-none border px-2 py-0.5 text-[10px] uppercase tracking-wider transition-colors hover:border-border-focus hover:bg-bg-active',
      ROLE_BADGE_CLASSES[role]
    )}
  >
    {ROLE_LABELS[role]}
  </span>
);

interface RoleMenuProps {
  columnName: string;
  columnSlug: string;
  activeRoles: TaskBoardColumnRole[];
  disabled: boolean;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
}

const RoleMenu = ({
  columnName,
  columnSlug,
  activeRoles,
  disabled,
  onSetColumnRole
}: RoleMenuProps): JSX.Element => {
  const open = useSignal(false);

  useEffect(() => {
    if (!open.value) return;

    const close = () => {
      open.value = false;
    };

    window.addEventListener('click', close);

    return () => {
      window.removeEventListener('click', close);
    };
  }, [open, open.value]);

  const toggleOpen = useCallback(
    (event: JSX.TargetedMouseEvent<HTMLButtonElement>) => {
      event.preventDefault();
      event.stopPropagation();
      open.value = !open.value;
    },
    [open]
  );

  const setRole = useCallback(
    (event: JSX.TargetedMouseEvent<HTMLButtonElement>, role: TaskBoardColumnRole) => {
      event.preventDefault();
      event.stopPropagation();
      const clearRole = role === 'review' && activeRoles.includes(role);
      onSetColumnRole(clearRole ? null : columnSlug, role);
      open.value = false;
    },
    [activeRoles, columnSlug, onSetColumnRole, open]
  );

  return (
    <div class='relative'>
      <Button
        type='button'
        variant='ghost'
        aria-label={`Column role settings for ${columnName}`}
        aria-expanded={open.value}
        disabled={disabled}
        onClick={toggleOpen}
        class='h-8 w-8 justify-center border border-border-base p-0 hover:border-border-focus'
      >
        <IconSettings size={15} stroke={1.75} aria-hidden='true' />
      </Button>
      {open.value && (
        <div
          role='menu'
          aria-label={`Column roles for ${columnName}`}
          class='absolute top-10 right-0 z-20 flex min-w-56 flex-col border border-border-base bg-bg-card p-1 shadow-modal'
          onClick={(event) => event.stopPropagation()}
        >
          {ROLE_ORDER.map((role) => {
            const active = activeRoles.includes(role);
            const requiredActive = active && role !== 'review';

            return (
              <Button
                key={role}
                type='button'
                variant='ghost'
                disabled={disabled || requiredActive}
                onClick={(event) => setRole(event, role)}
                class={clsx(
                  'flex-col items-start gap-1 px-2 py-2 text-left text-[10px]',
                  active && 'text-text-primary'
                )}
              >
                <span class='flex w-full items-center justify-between gap-3'>
                  <span>
                    {active && role === 'review' ? 'Clear' : 'Set'} {ROLE_LABELS[role]}
                  </span>
                  {active && <span class='text-accent'>Active</span>}
                </span>
                <span class='text-[10px] normal-case tracking-normal text-text-muted'>
                  {ROLE_HELP[role]}
                </span>
              </Button>
            );
          })}
        </div>
      )}
    </div>
  );
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
  onDragOverStatus,
  onDragEnd,
  onDropOnStatus,
  onSetColumnRole,
  onLoadMoreColumn,
  onColumnScroll,
  configuringColumns,
  dropPreviewColumn
}: TaskBoardColumnProps): JSX.Element => {
  const visibleTasks = column.tasks.slice(0, visibleCount);
  const hasMoreTasks = visibleTasks.length < column.tasks.length;
  const activeRoles = ROLE_ORDER.filter((role) => columnRoleActive(role, column.slug, columnRoles));
  const dropPreviewActive = dropPreviewColumn === column.slug;
  const dropOnStatus = useCallback(
    (event: JSX.TargetedDragEvent<HTMLElement>) => {
      onDropOnStatus(event as unknown as DragEvent, column.slug);
    },
    [column.slug, onDropOnStatus]
  );

  const dragOverStatus = useCallback(
    (event: JSX.TargetedDragEvent<HTMLElement>) => {
      onDragOverStatus(event as unknown as DragEvent, column.slug);
    },
    [column.slug, onDragOverStatus]
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
      onDragOver={dragOverStatus}
      onDrop={dropOnStatus}
      class={clsx(
        'flex min-h-80 flex-col gap-4 border bg-bg-card p-4 transition-colors duration-fast',
        dropPreviewActive ? 'border-accent bg-bg-hover/60 shadow-modal' : 'border-border-base'
      )}
    >
      <div class='flex flex-col gap-3 border-b border-border-base pb-3'>
        <div class='flex items-start justify-between gap-3'>
          <div class='flex flex-col gap-2'>
            <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
              {column.name}
            </h3>
            <div class='flex flex-wrap gap-1'>
              {activeRoles.length > 0 ? (
                activeRoles.map((role) => <RoleBadge key={role} role={role} />)
              ) : (
                <span class='select-none text-[10px] uppercase tracking-wider text-text-muted'>
                  No role
                </span>
              )}
            </div>
          </div>
          <div class='flex shrink-0 items-center gap-2'>
            <span class='text-xs tabular-nums text-text-muted'>{column.tasks.length}</span>
            <RoleMenu
              columnName={column.name}
              columnSlug={column.slug}
              activeRoles={activeRoles}
              disabled={configuringColumns}
              onSetColumnRole={onSetColumnRole}
            />
          </div>
        </div>
      </div>
      <div class='flex max-h-[70vh] flex-col gap-3 overflow-auto pr-1' onScroll={scrollColumn}>
        {dropPreviewActive && (
          <div class='border border-accent bg-bg-active p-4 text-xs font-medium text-text-primary'>
            Drop here to move into {column.name}.
          </div>
        )}
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
              onDragEnd={onDragEnd}
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
