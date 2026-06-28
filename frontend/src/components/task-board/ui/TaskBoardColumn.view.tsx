import { IconSettings } from '@tabler/icons-react';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import type { TaskBoardColumnData, TaskBoardColumnRole, TaskBoardRoleMenuData } from '../types';
import { TaskCard } from './TaskCard.view';

interface TaskBoardColumnProps {
  data: TaskBoardColumnData;
}

interface RoleBadgeProps {
  role: TaskBoardColumnRole;
}

interface RoleMenuProps {
  data: TaskBoardRoleMenuData;
}

const ROLE_LABELS: Record<TaskBoardColumnRole, string> = {
  pickup: 'Pickup',
  progress: 'In progress',
  done: 'Done',
  review: 'Review'
};

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

const RoleMenu = ({ data }: RoleMenuProps): JSX.Element => (
  <div class='relative'>
    <Button
      type='button'
      variant='ghost'
      aria-label={data.buttonLabel}
      title='Set this column as pickup, in progress, review pickup, or done'
      aria-expanded={data.open}
      disabled={data.disabled}
      onClick={data.onToggle}
      class='h-8 w-8 justify-center border border-border-base p-0 hover:border-border-focus'
    >
      <IconSettings size={15} stroke={1.75} aria-hidden='true' />
    </Button>
    {data.open && (
      <div
        role='menu'
        aria-label={data.menuLabel}
        class='absolute top-10 right-0 z-20 flex min-w-56 flex-col border border-border-base bg-bg-card p-1 shadow-modal'
        onClick={data.onStopClick}
      >
        {data.items.map((item) => (
          <Button
            key={item.role}
            type='button'
            variant='ghost'
            disabled={item.disabled}
            onClick={item.onClick}
            class={clsx(
              'flex-col items-start gap-1 px-2 py-2 text-left text-[10px]',
              item.active && 'text-text-primary'
            )}
          >
            <span class='flex w-full items-center justify-between gap-3'>
              <span>{item.label}</span>
              {item.active && <span class='text-accent'>Active</span>}
            </span>
            <span class='text-[10px] normal-case tracking-normal text-text-muted'>{item.help}</span>
          </Button>
        ))}
      </div>
    )}
  </div>
);

export const TaskBoardColumn = ({ data }: TaskBoardColumnProps): JSX.Element => (
  <section
    role='list'
    onDragOver={data.onDragOver}
    onDrop={data.onDrop}
    class={clsx(
      'flex min-h-80 flex-col gap-4 border bg-bg-card p-4 transition-colors duration-fast',
      data.dropPreviewActive ? 'border-accent bg-bg-hover/60 shadow-modal' : 'border-border-base'
    )}
  >
    <div class='flex flex-col gap-3 border-b border-border-base pb-3'>
      <div class='flex items-start justify-between gap-3'>
        <div class='flex flex-col gap-2'>
          <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
            {data.column.name}
          </h3>
          <div class='flex flex-wrap gap-1'>
            {data.activeRoles.length > 0 ? (
              data.activeRoles.map((role) => <RoleBadge key={role.role} role={role.role} />)
            ) : (
              <span class='select-none text-[10px] uppercase tracking-wider text-text-muted'>
                No role
              </span>
            )}
          </div>
        </div>
        <div class='flex shrink-0 items-center gap-2'>
          <span class='text-xs tabular-nums text-text-muted'>{data.taskCount}</span>
          <RoleMenu data={data.roleMenu} />
        </div>
      </div>
    </div>
    <div class='flex max-h-[70vh] flex-col gap-3 overflow-auto pr-1' onScroll={data.onScroll}>
      {data.dropPreviewActive && (
        <div class='border border-accent bg-bg-active p-4 text-xs font-medium text-text-primary shadow-card'>
          Drop to move into {data.column.name}.
        </div>
      )}
      {data.visibleTasks.length ? (
        data.visibleTasks.map((task) => <TaskCard key={task.task.id} data={task} />)
      ) : (
        <p class='border border-dashed border-border-base p-4 text-xs text-text-muted'>
          Drop tasks here or create a new one for this column.
        </p>
      )}
      {data.hasMoreTasks && (
        <Button
          type='button'
          variant='ghost'
          onClick={data.onLoadMore}
          class='border border-border-base'
        >
          Load more
        </Button>
      )}
    </div>
  </section>
);
