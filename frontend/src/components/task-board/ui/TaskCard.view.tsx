import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardTask } from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';

interface TaskMoveButtonProps {
  taskId: string;
  status: SelectOption;
  moving: boolean;
  onMoveTask: (taskId: string, status: string) => void;
}

interface TaskCardProps {
  task: TaskBoardTask;
  statusOptions: SelectOption[];
  moving: boolean;
  menuOpen: boolean;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onDragStart: (taskId: string) => void;
}

const TaskMoveButton = ({
  taskId,
  status,
  moving,
  onMoveTask
}: TaskMoveButtonProps): JSX.Element => {
  const moveTask = useCallback(
    (event: JSX.TargetedMouseEvent<HTMLButtonElement>) => {
      event.stopPropagation();
      onMoveTask(taskId, status.value);
    },
    [onMoveTask, status.value, taskId]
  );

  return (
    <Button
      type='button'
      variant='ghost'
      disabled={moving}
      onClick={moveTask}
      class='justify-start px-2 py-1 text-[10px]'
    >
      Mark {status.label}
    </Button>
  );
};

export const TaskCard = ({
  task,
  statusOptions,
  moving,
  menuOpen,
  onMoveTask,
  onOpenTask,
  onOpenTaskMenu,
  onDragStart
}: TaskCardProps): JSX.Element => {
  const startDrag = useCallback(() => {
    onDragStart(task.id);
  }, [onDragStart, task.id]);

  const openTask = useCallback(() => {
    onOpenTask(task);
  }, [onOpenTask, task]);

  const openMenu = useCallback(
    (event: JSX.TargetedMouseEvent<HTMLElement>) => {
      onOpenTaskMenu(event as unknown as MouseEvent, task.id);
    },
    [onOpenTaskMenu, task.id]
  );

  const openTaskFromKeyboard = useCallback(
    (event: JSX.TargetedKeyboardEvent<HTMLElement>) => {
      if (event.key !== 'Enter' && event.key !== ' ') return;

      event.preventDefault();
      onOpenTask(task);
    },
    [onOpenTask, task]
  );

  return (
    <article
      draggable
      onDragStart={startDrag}
      onClick={openTask}
      onContextMenu={openMenu}
      onKeyDown={openTaskFromKeyboard}
      class='relative flex cursor-pointer flex-col gap-3 border border-border-base bg-bg-input p-4 text-left transition-colors hover:border-border-focus focus-visible:border-border-focus focus-visible:outline-none'
    >
      <div class='flex items-start justify-between gap-3'>
        <div class='flex flex-col gap-1'>
          <span class='text-[10px] uppercase tracking-wider text-text-muted'>
            {task.number ? `#${task.number}` : task.id.slice(0, 8)}
          </span>
          <h3 class='text-sm font-medium text-text-primary'>{task.title}</h3>
        </div>
        <span class='border border-border-base px-2 py-1 text-[10px] uppercase tracking-wider text-text-muted'>
          {task.priority}
        </span>
      </div>

      <div class='flex items-center justify-between gap-3 text-[11px] text-text-muted'>
        <span>{task.assigneeName ?? 'Unassigned'}</span>
        <span>{new Date(task.createdAt).toLocaleDateString()}</span>
      </div>

      {menuOpen && (
        <div class='absolute top-3 right-3 z-10 flex min-w-36 flex-col border border-border-base bg-bg-card p-1 shadow-modal'>
          {statusOptions
            .filter((option) => option.value !== task.status)
            .map((option) => (
              <TaskMoveButton
                key={option.value}
                taskId={task.id}
                status={option}
                moving={moving}
                onMoveTask={onMoveTask}
              />
            ))}
        </div>
      )}
    </article>
  );
};
