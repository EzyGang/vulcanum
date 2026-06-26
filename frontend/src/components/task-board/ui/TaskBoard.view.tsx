import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoard, TaskBoardTask } from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Select } from '../../shared/ui/Select.view';
import { TextArea } from '../../shared/ui/TextArea.view';

interface TaskBoardViewProps {
  data: {
    selectedProjectKey: string | null;
    board?: TaskBoard;
    statusOptions: SelectOption[];
    repoItems: SelectOption[];
    selectedRepoNames: string[];
    selectedTask: TaskBoardTask | null;
  };
  form: {
    title: string;
    body: string;
    status: string;
    createError: string | null;
    serverError: string | null;
  };
  status: {
    loading: boolean;
    error: string | null;
    creating: boolean;
    movingTaskId: string | null;
    moving: boolean;
    reposLoading: boolean;
    connectingRepos: boolean;
    connected: boolean;
  };
  actions: {
    onTitleInput: (event: Event) => void;
    onBodyInput: (event: Event) => void;
    onStatusChange: (status: string) => void;
    onSubmitTask: (event: Event) => void;
    onMoveTask: (taskId: string, status: string) => void;
    onToggleRepo: (repoFullName: string) => void;
    onOpenTask: (task: TaskBoardTask) => void;
    onCloseTask: () => void;
    onDragStart: (taskId: string) => void;
    onDragOver: (event: DragEvent) => void;
    onDropOnStatus: (event: DragEvent, status: string) => void;
  };
}

interface TaskCardProps {
  task: TaskBoardTask;
  statusOptions: SelectOption[];
  moving: boolean;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onDragStart: (taskId: string) => void;
}

const TaskCard = ({
  task,
  statusOptions,
  moving,
  onMoveTask,
  onOpenTask,
  onDragStart
}: TaskCardProps): JSX.Element => (
  <article
    draggable
    onDragStart={() => onDragStart(task.id)}
    onClick={() => onOpenTask(task)}
    class='flex cursor-pointer flex-col gap-3 border border-border-base bg-bg-input p-4 transition-colors hover:border-border-focus'
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

    {task.description && <p class='text-xs leading-5 text-text-secondary'>{task.description}</p>}

    <div class='flex items-center justify-between gap-3 text-[11px] text-text-muted'>
      <span>{task.assigneeName ?? 'Unassigned'}</span>
      <span>{new Date(task.createdAt).toLocaleDateString()}</span>
    </div>

    <div class='flex flex-wrap gap-2'>
      {statusOptions
        .filter((option) => option.value !== task.status)
        .map((option) => (
          <Button
            key={option.value}
            type='button'
            variant='ghost'
            disabled={moving}
            onClick={(event) => {
              event.stopPropagation();
              onMoveTask(task.id, option.value);
            }}
            class='border border-border-base px-2 py-1 text-[10px] hover:bg-bg-hover'
          >
            Mark {option.label}
          </Button>
        ))}
    </div>
  </article>
);

export const TaskBoardView = ({
  data: { selectedProjectKey, board, statusOptions, repoItems, selectedRepoNames, selectedTask },
  form,
  status,
  actions
}: TaskBoardViewProps): JSX.Element => {
  if (!selectedProjectKey) {
    return (
      <EmptyState
        title='No project selected'
        description='Connect a task provider in settings, then select a project from the navbar.'
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

  return (
    <div class='flex flex-col gap-6 animate-fade-in'>
      <div class='flex flex-col gap-2'>
        <span class='text-xs uppercase tracking-wider text-accent'>Task provider board</span>
        <h2 class='text-3xl font-semibold text-text-primary'>{board.project.name}</h2>
        <p class='text-sm text-text-muted'>
          Proxy view for provider columns, task creation, status movement, and project repository
          assignment.
        </p>
      </div>

      <section class='flex flex-col gap-4 border border-border-base bg-bg-card p-5'>
        <div class='flex items-center justify-between gap-3'>
          <div class='flex flex-col gap-1'>
            <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
              Connected repositories
            </h3>
            <p class='text-xs text-text-muted'>
              Assign GitHub repositories related to this provider project.
            </p>
          </div>
          <span class='text-xs uppercase tracking-wider text-text-muted'>
            {status.connected ? 'Connected' : 'Not connected'}
          </span>
        </div>
        {status.reposLoading && <p class='text-xs text-text-muted'>Loading repositories…</p>}
        {!status.reposLoading && repoItems.length === 0 && (
          <p class='text-xs text-text-muted'>No GitHub repositories are available.</p>
        )}
        {repoItems.length > 0 && (
          <div class='grid max-h-48 gap-2 overflow-auto border border-border-base bg-bg-input p-3 md:grid-cols-2'>
            {repoItems.map((repo) => (
              <label
                key={repo.value}
                for={`repo-${repo.value}`}
                class='flex items-center gap-2 text-sm text-text-secondary'
              >
                <Checkbox
                  id={`repo-${repo.value}`}
                  checked={selectedRepoNames.includes(repo.value)}
                  disabled={status.connectingRepos}
                  onCheckedChange={() => actions.onToggleRepo(repo.value)}
                />
                <span>{repo.label}</span>
              </label>
            ))}
          </div>
        )}
      </section>

      <form
        onSubmit={actions.onSubmitTask}
        class='grid gap-4 border border-border-base bg-bg-card p-5 lg:grid-cols-[1fr_2fr_auto]'
      >
        <div class='flex flex-col gap-2'>
          <label class='text-xs uppercase tracking-wider text-text-muted' for='task-title'>
            Title
          </label>
          <Input
            id='task-title'
            value={form.title}
            onInput={actions.onTitleInput}
            placeholder='Ship the proxy board'
            invalid={Boolean(form.createError)}
          />
        </div>
        <div class='flex flex-col gap-2'>
          <label class='text-xs uppercase tracking-wider text-text-muted' for='task-body'>
            Body
          </label>
          <TextArea
            id='task-body'
            value={form.body}
            onInput={actions.onBodyInput}
            rows={3}
            placeholder='Task details for whoever picks this up'
          />
        </div>
        <div class='flex min-w-48 flex-col gap-2'>
          <label class='text-xs uppercase tracking-wider text-text-muted' for='task-status'>
            Column
          </label>
          <Select
            id='task-status'
            items={statusOptions}
            value={form.status}
            onValueChange={actions.onStatusChange}
            placeholder='First column'
            disabled={!statusOptions.length}
          />
          <Button type='submit' variant='primary' disabled={status.creating} class='mt-auto'>
            {status.creating ? 'Creating…' : 'Create task'}
          </Button>
        </div>
      </form>

      {(form.createError || form.serverError) && (
        <ErrorBanner message={form.createError ?? form.serverError ?? 'Unable to update board'} />
      )}

      <div class='grid gap-4 xl:grid-cols-3'>
        {board.columns.map((column) => (
          <section
            key={column.id}
            role='list'
            onDragOver={actions.onDragOver}
            onDrop={(event) => actions.onDropOnStatus(event, column.slug)}
            class='flex min-h-80 flex-col gap-4 border border-border-base bg-bg-card p-4'
          >
            <div class='flex items-center justify-between gap-3 border-b border-border-base pb-3'>
              <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
                {column.name}
              </h3>
              <span class='text-xs tabular-nums text-text-muted'>{column.tasks.length}</span>
            </div>
            <div class='flex flex-col gap-3'>
              {column.tasks.length ? (
                column.tasks.map((task) => (
                  <TaskCard
                    key={task.id}
                    task={task}
                    statusOptions={statusOptions}
                    moving={status.moving && status.movingTaskId === task.id}
                    onMoveTask={actions.onMoveTask}
                    onOpenTask={actions.onOpenTask}
                    onDragStart={actions.onDragStart}
                  />
                ))
              ) : (
                <p class='border border-dashed border-border-base p-4 text-xs text-text-muted'>
                  Drop tasks here or create a new one for this column.
                </p>
              )}
            </div>
          </section>
        ))}
      </div>

      <Dialog open={Boolean(selectedTask)} onOpenChange={(open) => !open && actions.onCloseTask()}>
        <Dialog.Portal>
          <Dialog.Backdrop />
          <Dialog.Popup class='flex w-[min(92vw,640px)] flex-col gap-5'>
            {selectedTask && (
              <>
                <div class='flex items-start justify-between gap-4'>
                  <div class='flex flex-col gap-2'>
                    <Dialog.Title>{selectedTask.title}</Dialog.Title>
                    <Dialog.Description>
                      {selectedTask.number ? `#${selectedTask.number}` : selectedTask.id}
                    </Dialog.Description>
                  </div>
                  <Dialog.Close>
                    <Button type='button' variant='ghost'>
                      Close
                    </Button>
                  </Dialog.Close>
                </div>
                <div class='grid gap-3 text-sm text-text-secondary md:grid-cols-2'>
                  <span>Status: {selectedTask.status}</span>
                  <span>Priority: {selectedTask.priority}</span>
                  <span>Assignee: {selectedTask.assigneeName ?? 'Unassigned'}</span>
                  <span>Created: {new Date(selectedTask.createdAt).toLocaleString()}</span>
                </div>
                {selectedTask.description ? (
                  <p class='whitespace-pre-wrap border border-border-base bg-bg-input p-4 text-sm leading-6 text-text-secondary'>
                    {selectedTask.description}
                  </p>
                ) : (
                  <p class='border border-border-base bg-bg-input p-4 text-sm text-text-muted'>
                    No task body.
                  </p>
                )}
                <div class='flex flex-wrap gap-2'>
                  {statusOptions
                    .filter((option) => option.value !== selectedTask.status)
                    .map((option) => (
                      <Button
                        key={option.value}
                        type='button'
                        variant='secondary'
                        disabled={status.moving}
                        onClick={() => actions.onMoveTask(selectedTask.id, option.value)}
                      >
                        Move to {option.label}
                      </Button>
                    ))}
                </div>
              </>
            )}
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog>
    </div>
  );
};
