import { IconArrowLeft, IconArrowRight, IconEyeOff, IconSettings } from '@tabler/icons-react';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { ROLE_HELP, ROLE_LABELS } from '../hooks/taskBoardViewModel.support';
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

const ROLE_BADGE_CLASSES: Record<TaskBoardColumnRole, string> = {
  pickup: 'border-warning-border bg-warning-bg text-warning',
  progress: 'border-accent/60 bg-accent/10 text-accent-light',
  done: 'border-success-border bg-success-bg text-success'
};

const ACTION_BUTTON_CLASS =
  'h-8 w-8 justify-center border border-border-base bg-bg-panel p-0 text-text-muted transition-[color,background-color,border-color,transform] hover:border-border-focus hover:bg-bg-hover hover:text-text-primary active:scale-[0.96]';

const RoleBadge = ({ role }: RoleBadgeProps): JSX.Element => (
  <span
    title={ROLE_HELP[role]}
    class={clsx(
      'inline-flex h-6 select-none items-center border px-2 text-[10px] font-medium leading-none uppercase tracking-[0.12em] transition-colors hover:border-border-focus hover:bg-bg-active',
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
      title='Set this column as pickup, in progress, or done'
      aria-expanded={data.open}
      disabled={data.disabled}
      onClick={data.onToggle}
      class={ACTION_BUTTON_CLASS}
    >
      <IconSettings size={14} stroke={1.75} aria-hidden='true' />
    </Button>
    {data.open && (
      <div
        role='menu'
        aria-label={data.menuLabel}
        class='absolute top-9 right-0 z-20 flex min-w-56 flex-col border border-border-base bg-bg-card p-1 shadow-modal'
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
      'flex min-h-80 min-w-0 flex-col gap-4 border bg-bg-card p-4 transition-colors duration-fast',
      data.dropPreviewActive ? 'border-accent bg-bg-hover/60' : 'border-border-base'
    )}
  >
    <div class='flex flex-col gap-2 border-b border-border-base pb-3'>
      <div class='flex min-w-0 items-start gap-2'>
        <h3 class='min-w-0 flex-1 text-balance text-[0.82rem] font-semibold leading-snug tracking-[0.14em] text-text-primary uppercase'>
          {data.column.name}
        </h3>
        <span class='inline-flex h-6 min-w-6 shrink-0 items-center justify-center border border-border-base bg-bg-panel px-1.5 text-[11px] leading-none tabular-nums text-text-secondary'>
          {data.taskCount}
        </span>
      </div>
      <div class='flex min-w-0 flex-wrap items-center gap-1.5'>
        {data.activeRoles.length > 0 ? (
          data.activeRoles.map((role) => <RoleBadge key={role.role} role={role.role} />)
        ) : (
          <span class='inline-flex h-6 select-none items-center border border-dashed border-border-base px-2 text-[10px] font-medium leading-none tracking-[0.12em] text-text-muted uppercase'>
            No role
          </span>
        )}
        <div class='flex shrink-0 items-center gap-1.5'>
          <div
            role='group'
            class='flex items-center gap-1.5'
            aria-label={`View controls for ${data.column.name}`}
          >
            <Button
              type='button'
              variant='ghost'
              aria-label={`Move ${data.column.name} column left`}
              title={`Move ${data.column.name} column left`}
              disabled={!data.viewControls.canMoveLeft}
              onClick={data.viewControls.onMoveLeft}
              class={clsx(
                ACTION_BUTTON_CLASS,
                'disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100'
              )}
            >
              <IconArrowLeft size={14} stroke={1.75} aria-hidden='true' />
            </Button>
            <Button
              type='button'
              variant='ghost'
              aria-label={`Move ${data.column.name} column right`}
              title={`Move ${data.column.name} column right`}
              disabled={!data.viewControls.canMoveRight}
              onClick={data.viewControls.onMoveRight}
              class={clsx(
                ACTION_BUTTON_CLASS,
                'disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100'
              )}
            >
              <IconArrowRight size={14} stroke={1.75} aria-hidden='true' />
            </Button>
            <Button
              type='button'
              variant='ghost'
              aria-label={`Hide ${data.column.name} column`}
              title={`Hide ${data.column.name} column`}
              onClick={data.viewControls.onHide}
              class={ACTION_BUTTON_CLASS}
            >
              <IconEyeOff size={14} stroke={1.75} aria-hidden='true' />
            </Button>
          </div>
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
