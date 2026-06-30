import { IconDots } from '@tabler/icons-react';
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
    class='justify-start px-2 py-1 text-left text-[10px] whitespace-nowrap'
  >
    Mark {action.label}
  </Button>
);

export const TaskCard = ({ data }: TaskCardProps): JSX.Element => {
  const menuStyle = data.menuPosition
    ? {
        left: `${data.menuPosition.x}px`,
        top: `${data.menuPosition.y}px`
      }
    : undefined;

  return (
    <article
      draggable
      onClick={data.onClick}
      onDragStart={data.onDragStart}
      onDragEnd={data.onDragEnd}
      onKeyDown={data.onKeyDown}
      class='relative flex cursor-pointer flex-col gap-3 rounded border border-border-base bg-bg-input p-4 text-left transition-colors hover:border-border-focus focus-visible:border-border-focus focus-visible:outline-none'
    >
      <div class='flex items-start justify-between gap-3'>
        <span class='text-xs font-semibold uppercase tracking-wide text-text-muted'>
          {data.displayId}
        </span>
        <Button
          type='button'
          variant='ghost'
          disabled={data.moving}
          aria-label={`Actions for ${data.task.title}`}
          class='-mr-2 -mt-2'
          onClick={data.onOpenMenu}
        >
          <IconDots size={16} />
        </Button>
      </div>
      <h3 class='text-sm font-semibold text-text-primary'>{data.task.title}</h3>
      <div class='flex flex-wrap gap-1'>
        {data.task.labels.map((label) => (
          <span
            key={label.id}
            class='rounded-full px-2 py-0.5 text-xs font-medium'
            style={{ backgroundColor: label.color, color: '#0b1020' }}
          >
            {label.name}
          </span>
        ))}
      </div>
      <div class='flex items-center justify-between gap-3 text-[11px] text-text-muted'>
        <span>{data.createdAtLabel}</span>
        {data.moving && <span>Moving…</span>}
      </div>

      {data.menuOpen && (
        <div
          role='menu'
          class='fixed z-50 flex max-h-72 w-56 flex-col overflow-y-auto rounded border border-border-base bg-bg-card p-1 shadow-modal'
          style={menuStyle}
          onClick={data.onStopMenuClick}
        >
          {data.moveActions.map((action) => (
            <TaskMoveButton key={action.value} action={action} moving={data.moving} />
          ))}
        </div>
      )}
    </article>
  );
};
