import {
  IconAlertTriangle,
  IconBolt,
  IconInfoCircle,
  IconSettings,
  IconX
} from '@tabler/icons-react';
import { clsx } from 'clsx';
import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { TaskBoardViewProps } from '../types';
import { TaskBoardColumn } from './TaskBoardColumn.view';
import { TaskBoardSettingsDialog } from './TaskBoardSettingsDialog.view';
import { TaskCreateDialog } from './TaskCreateDialog.view';
import { TaskDetailsDialog } from './TaskDetailsDialog.view';

const LIFECYCLE_LABELS = [
  {
    color: '#2563EB',
    name: 'Implementation running',
    description: 'Vulcanum has started an implementation job for this ticket.'
  },
  {
    color: '#D97706',
    name: 'Review needed',
    description: 'Implementation produced work, but no automated review is currently running.'
  },
  {
    color: '#7C3AED',
    name: 'Review running',
    description: 'Vulcanum has queued or started an automated review for this ticket.'
  },
  {
    color: '#DC2626',
    name: 'Needs attention',
    description: 'Automation failed or blocked and will not continue without a human fix.'
  },
  {
    color: '#16A34A',
    name: 'Ready for human',
    description: 'Implementation and review are complete, and the ticket is in its final column.'
  }
];
export const TaskBoardView = ({
  data: {
    selectedProjectKey,
    board,
    boardColumnCount,
    columns,
    helpCards,
    dismissedHelpCards,
    automationLabel,
    statusOptions,
    selectedTask,
    availableLabels,
    selectedTaskCreatedAtLabel,
    selectedTaskMoveActions,
    createDialogOpen,
    settingsDialogOpen,
    automationEnabled,
    repositorySettings,
    projectSettings,
    reviewSettings
  },
  form,
  status,
  actions
}: TaskBoardViewProps): JSX.Element => {
  if (!selectedProjectKey) {
    return (
      <EmptyState
        title='Select a board to begin'
        description='Use the board picker in the navigation to choose a connected provider project.'
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
      <EmptyState
        title='Board unavailable'
        description='The selected provider project did not return board data.'
      />
    );
  }

  const showMissingRepoWarning = automationEnabled && !repositorySettings.hasSelectedRepos;

  return (
    <div class='flex flex-col gap-6 animate-fade-in'>
      <div class='flex flex-col gap-4 md:flex-row md:items-start md:justify-between'>
        <div class='flex min-w-0 flex-col gap-2'>
          <span class='text-xs uppercase tracking-wider text-accent'>Task provider board</span>
          <h2 class='text-3xl font-semibold text-text-primary'>{board.project.name}</h2>
          <div class='max-w-3xl text-sm leading-relaxed text-text-muted'>
            <p>
              Use this board as a task-provider proxy: create tickets, move them through provider
              columns, and turn worker automation on only after pickup/progress/done roles are
              mapped.{' '}
              <span class='group relative inline-flex align-middle text-text-muted hover:text-text-primary focus-within:text-text-primary'>
                <button
                  type='button'
                  class='inline-flex cursor-help'
                  aria-label='Board sync details'
                >
                  <IconInfoCircle size={16} stroke={1.75} aria-hidden='true' />
                </button>
                <span class='pointer-events-none absolute top-6 left-0 z-20 hidden w-[min(80vw,48rem)] border border-border-base bg-bg-card px-3 py-2 text-xs leading-relaxed text-text-secondary shadow-modal group-focus-within:block group-hover:block'>
                  <span class='block font-medium text-text-primary'>Proxy view</span>
                  <span class='mt-1 block'>
                    Vulcanum reads projects, columns, and tasks from the connected provider, then
                    writes task creation and status changes back through that same provider API.
                  </span>
                  <span class='mt-1 block'>
                    The board refreshes periodically, so provider-side edits still show up here.
                  </span>
                </span>
              </span>
            </p>
          </div>
        </div>
        <div class='flex shrink-0 flex-wrap items-center gap-2 md:justify-end'>
          <button
            type='button'
            onClick={actions.onToggleAutomation}
            disabled={!status.connected || status.savingAutomation}
            aria-pressed={automationEnabled}
            aria-label={automationEnabled ? 'Turn automation off' : 'Turn automation on'}
            class={clsx(
              'inline-flex h-10 items-center gap-2 border px-3 text-xs font-medium uppercase tracking-wider transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus disabled:cursor-not-allowed disabled:opacity-50',
              automationEnabled
                ? 'border-accent/70 bg-bg-active text-accent hover:border-accent'
                : 'border-border-base bg-bg-panel text-text-muted hover:bg-bg-hover hover:text-text-primary'
            )}
          >
            <IconBolt size={15} stroke={1.75} aria-hidden='true' />
            {status.savingAutomation ? `Saving ${automationLabel.toLowerCase()}…` : automationLabel}
          </button>
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

      {showMissingRepoWarning && (
        <section
          role='alert'
          class='flex flex-col gap-3 border border-warning/40 bg-warning-bg p-4 text-sm text-text-secondary md:flex-row md:items-center md:justify-between'
        >
          <div class='flex items-start gap-3'>
            <IconAlertTriangle
              size={18}
              stroke={1.8}
              class='mt-0.5 shrink-0 text-warning'
              aria-hidden='true'
            />
            <div class='flex flex-col gap-1'>
              <p class='font-medium text-warning'>Automation has no repositories set</p>
              <p>
                Select at least one repository for this project before relying on automation.
                Workers need pinned repos to clone code and resolve PR review targets.
              </p>
            </div>
          </div>
          <Button
            type='button'
            variant='ghost'
            onClick={actions.onOpenSettings}
            class='shrink-0 border border-warning/40 text-warning hover:border-warning hover:bg-warning-bg'
          >
            Set repositories
          </Button>
        </section>
      )}

      {helpCards.length > 0 && (
        <section class='grid grid-cols-1 gap-3 md:grid-cols-3'>
          {helpCards.map((card) => (
            <article
              key={card.id}
              class='group relative border border-border-base bg-bg-card p-4 pr-12 shadow-card transition-colors hover:border-border-focus'
            >
              <p class='text-xs font-medium uppercase tracking-wider text-accent'>{card.title}</p>
              <p class='mt-2 text-sm leading-relaxed text-text-secondary'>{card.body}</p>
              <button
                type='button'
                aria-label={`Dismiss ${card.title} help`}
                onClick={card.onDismiss}
                class='absolute top-3 right-3 inline-flex size-7 items-center justify-center border border-transparent text-text-muted transition-colors hover:border-border-base hover:bg-bg-hover hover:text-text-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus'
              >
                <IconX size={14} stroke={1.75} aria-hidden='true' />
              </button>
            </article>
          ))}
        </section>
      )}

      {!dismissedHelpCards.includes('lifecycle-labels') && (
        <section class='relative border border-border-base bg-bg-card p-4 pr-12'>
          <button
            type='button'
            aria-label='Dismiss Lifecycle labels help'
            onClick={() => actions.onDismissHelpCard('lifecycle-labels')}
            class='absolute top-3 right-3 inline-flex size-7 items-center justify-center border border-transparent text-text-muted transition-colors hover:border-border-base hover:bg-bg-hover hover:text-text-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus'
          >
            <IconX size={14} stroke={1.75} aria-hidden='true' />
          </button>
          <div class='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
            <div class='flex max-w-2xl flex-col gap-1'>
              <p class='text-xs font-medium uppercase tracking-wider text-accent'>
                Lifecycle labels
              </p>
              <p class='text-sm leading-relaxed text-text-secondary'>
                Vulcanum keeps one managed label active on each automated ticket so the board shows
                the current automation handoff without changing your manual provider labels.
              </p>
            </div>
            <p class='text-xs uppercase tracking-wider text-text-muted'>One active at a time</p>
          </div>
          <div class='mt-4 grid grid-cols-1 gap-2 md:grid-cols-5'>
            {LIFECYCLE_LABELS.map((label) => (
              <article key={label.name} class='border border-border-base bg-bg-panel p-3'>
                <div class='flex items-center gap-2'>
                  <span
                    class='size-2 shrink-0 border border-border-base'
                    style={{ background: label.color }}
                  />
                  <p class='text-[11px] font-medium uppercase tracking-wider text-text-primary'>
                    {label.name}
                  </p>
                </div>
                <p class='mt-2 text-xs leading-relaxed text-text-muted'>{label.description}</p>
              </article>
            ))}
          </div>
        </section>
      )}

      {(form.createError || form.serverError) && (
        <ErrorBanner message={form.createError ?? form.serverError ?? 'Unable to update board'} />
      )}

      <div
        class='grid grid-cols-1 gap-4 lg:grid-cols-[repeat(var(--board-column-count),minmax(0,1fr))]'
        style={`--board-column-count: ${boardColumnCount}`}
      >
        {columns.map((column) => (
          <TaskBoardColumn key={column.column.id} data={column} />
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
        repositorySettings={repositorySettings}
        projectSettings={projectSettings}
        reviewSettings={reviewSettings}
        status={status}
        actions={actions}
      />
      <TaskDetailsDialog
        createdAtLabel={selectedTaskCreatedAtLabel}
        task={selectedTask}
        availableLabels={availableLabels}
        form={form}
        moveActions={selectedTaskMoveActions}
        status={status}
        actions={actions}
      />
    </div>
  );
};
