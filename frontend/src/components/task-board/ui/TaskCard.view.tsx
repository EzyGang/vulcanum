import { IconMenu2 } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import type { TaskBoardMoveAction, TaskBoardTaskCardData } from '../types';

interface TaskMoveButtonProps {
  action: TaskBoardMoveAction;
  moving: boolean;
}

interface TaskCardProps {
  data: TaskBoardTaskCardData;
}

const TaskMoveButton = ({ action, moving }: TaskMoveButtonProps): JSX.Element => (
  <Button
    type='button'
    variant='ghost'
    disabled={moving}
    onClick={action.onClick}
    class='justify-start px-2 py-1 text-[10px]'
  >
    Mark {action.label}
  </Button>
);

export const TaskCard = ({ data }: TaskCardProps): JSX.Element => (
  <article
    draggable
    onDragStart={data.onDragStart}
    onDragEnd={data.onDragEnd}
    onClick={data.onClick}
    onContextMenu={data.onContextMenu}
    onKeyDown={data.onKeyDown}
    class='relative flex cursor-pointer flex-col gap-3 border border-border-base bg-bg-input p-4 text-left transition-colors hover:border-border-focus focus-visible:border-border-focus focus-visible:outline-none'
  >
    <div class='flex items-start justify-between gap-3'>
      <div class='flex flex-col gap-1'>
        <span class='text-[10px] uppercase tracking-wider text-text-muted'>{data.displayId}</span>
        <h3 class='text-sm font-medium text-text-primary'>{data.task.title}</h3>
      </div>
      <div class='flex shrink-0 items-center gap-2'>
        <span class='border border-border-base px-2 py-1 text-[10px] uppercase tracking-wider text-text-muted'>
          {data.task.priority}
        </span>
        <Button
          type='button'
          variant='ghost'
          aria-label={`Task actions for ${data.task.title}`}
          aria-expanded={data.menuOpen}
          onClick={data.onContextMenu}
          class='h-8 w-8 justify-center border border-border-base p-0'
        >
          <IconMenu2 size={16} stroke={1.75} aria-hidden='true' />
        </Button>
      </div>
    </div>

    {data.task.labels.length > 0 && (
      <div class='flex flex-wrap gap-1'>
        {data.task.labels.map((label) => (
          <span
            key={label.id}
            class='border border-border-base bg-bg-card px-2 py-0.5 text-[10px] uppercase tracking-wider text-text-secondary'
          >
            {label.name}
          </span>
        ))}
      </div>
    )}

    <div class='flex items-center justify-between gap-3 text-[11px] text-text-muted'>
      <span>{data.task.assigneeName ?? 'Unassigned'}</span>
      <span>{data.createdAtLabel}</span>
    </div>

    {data.menuOpen && (
      <div
        role='menu'
        aria-label={`Actions for ${data.task.title}`}
        class='absolute top-12 right-3 z-10 flex min-w-36 flex-col border border-border-base bg-bg-card p-1 shadow-modal'
        onClick={data.onStopMenuClick}
      >
        {data.moveActions.map((action) => (
          <TaskMoveButton key={action.value} action={action} moving={data.moving} />
        ))}
      </div>
    )}
  </article>
);
