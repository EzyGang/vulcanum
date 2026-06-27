import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { TaskBoardViewProps } from '../types';
import { SettingsIcon } from './SettingsIcon.view';
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
    visibleTaskCounts
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
          <p class='text-sm text-text-muted'>
            Proxy view for provider columns, task creation, status movement, and project repository
            assignment.
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
            <SettingsIcon />
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
        repoItems={repoItems}
        selectedRepoNames={selectedRepoNames}
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
