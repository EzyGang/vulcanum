import { IconInfoCircle, IconSettings } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { TaskBoardViewProps } from '../types';
import { TaskBoardColumn } from './TaskBoardColumn.view';
import { TaskBoardSettingsDialog } from './TaskBoardSettingsDialog.view';
import { TaskCreateDialog } from './TaskCreateDialog.view';
import { TaskDetailsDialog } from './TaskDetailsDialog.view';

export const TaskBoardView = ({
  data: {
    selectedProjectKey,
    board,
    statusOptions,
    repoItems,
    selectedRepoNames,
    selectedTask,
    createDialogOpen,
    settingsDialogOpen,
    actionMenuTaskId,
    visibleTaskCounts,
    columnRoles
  },
  form,
  status,
  actions
}: TaskBoardViewProps): JSX.Element => {
  if (!selectedProjectKey) {
    return (
      <EmptyState
        title='No project selected'
        description='Add a task provider project to Vulcanum, then select it from the navbar.'
      />
    );
  }

  if (status.loading) {
    return <p class='text-sm text-text-muted'>Loading board…</p>;
  }

  if (status.error) {
    return <ErrorBanner message={status.error} />;
  }

  if (!board) {
    return (
      <EmptyState title='Board unavailable' description='Select another project from the navbar.' />
    );
  }

  const boardColumnCount = Math.max(board.columns.length, 1);

  return (
    <div class='flex flex-col gap-6 animate-fade-in'>
      <div class='flex flex-col gap-4 md:flex-row md:items-start md:justify-between'>
        <div class='flex flex-col gap-2'>
          <span class='text-xs uppercase tracking-wider text-accent'>Task provider board</span>
          <h2 class='text-3xl font-semibold text-text-primary'>{board.project.name}</h2>
          <p class='flex items-center gap-2 text-sm text-text-muted'>
            <span>
              Provider-backed board for task creation, status movement, repository pinning, and
              automation column roles.
            </span>
            <span
              class='inline-flex text-text-muted hover:text-text-primary'
              title='Proxy view. Vulcanum sits on top of the connected task provider: actions update the provider, and the board refreshes periodically to pick up external changes.'
            >
              <IconInfoCircle size={16} stroke={1.75} aria-label='Board sync details' />
            </span>
          </p>
        </div>
        <div class='flex items-center gap-2'>
          <Button type='button' variant='primary' onClick={actions.onOpenCreateTask}>
            Create task
          </Button>
          <Button
            type='button'
            variant='ghost'
            aria-label='Board settings'
            onClick={actions.onOpenSettings}
            class='border border-border-base p-2'
          >
            <IconSettings size={16} stroke={1.75} aria-hidden='true' />
          </Button>
        </div>
      </div>

      {(form.createError || form.serverError) && (
        <ErrorBanner message={form.createError ?? form.serverError ?? 'Unable to update board'} />
      )}

      <div
        class='grid grid-cols-1 gap-4 lg:grid-cols-[repeat(var(--board-column-count),minmax(0,1fr))]'
        style={`--board-column-count: ${boardColumnCount}`}
      >
        {board.columns.map((column) => (
          <TaskBoardColumn
            key={column.id}
            column={column}
            visibleCount={visibleTaskCounts[column.slug] ?? 20}
            statusOptions={statusOptions}
            columnRoles={columnRoles}
            moving={status.moving}
            movingTaskId={status.movingTaskId}
            actionMenuTaskId={actionMenuTaskId}
            onMoveTask={actions.onMoveTask}
            onOpenTask={actions.onOpenTask}
            onOpenTaskMenu={actions.onOpenTaskMenu}
            onDragStart={actions.onDragStart}
            onDragOver={actions.onDragOver}
            onDropOnStatus={actions.onDropOnStatus}
            onLoadMoreColumn={actions.onLoadMoreColumn}
            onColumnScroll={actions.onColumnScroll}
          />
        ))}
      </div>

      <TaskCreateDialog
        open={createDialogOpen}
        form={form}
        status={status}
        statusOptions={statusOptions}
        actions={actions}
      />
      <TaskBoardSettingsDialog
        open={settingsDialogOpen}
        form={form.settings}
        repoItems={repoItems}
        selectedRepoNames={selectedRepoNames}
        columnRoles={columnRoles}
        statusOptions={statusOptions}
        status={status}
        actions={actions}
      />
      <TaskDetailsDialog
        task={selectedTask}
        statusOptions={statusOptions}
        status={status}
        actions={actions}
      />
    </div>
  );
};
