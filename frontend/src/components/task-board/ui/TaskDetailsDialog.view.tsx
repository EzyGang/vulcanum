import type { JSX } from 'preact';
import type {
  TaskBoardLabel,
  TaskBoardTask,
  TaskBoardTaskAugmentation
} from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import { formatTaskDisplayId } from '../hooks/taskBoardViewModel.support';
import type {
  TaskBoardActions,
  TaskBoardFormState,
  TaskBoardMoveAction,
  TaskBoardStatusState
} from '../types';
import { TaskUsageSummary } from './TaskUsageSummary.view';

interface TaskDetailsMoveButtonProps {
  action: TaskBoardMoveAction;
  moving: boolean;
}

interface TaskDetailsDialogProps {
  task: TaskBoardTask | null;
  availableLabels: TaskBoardLabel[];
  createdAtLabel: string | null;
  moveActions: TaskBoardMoveAction[];
  augmentation: TaskBoardTaskAugmentation | null;
  form: Pick<TaskBoardFormState, 'editTitle' | 'editBody' | 'editLabelIds' | 'editError'>;
  status: Pick<TaskBoardStatusState, 'moving' | 'updatingTask' | 'updatingTaskLabel'>;
  actions: Pick<
    TaskBoardActions,
    | 'onTaskDetailsOpenChange'
    | 'onEditTaskTitleInput'
    | 'onEditTaskBodyInput'
    | 'onSubmitTaskEdit'
    | 'onToggleTaskLabel'
    | 'onDeleteLabel'
  >;
}

const TaskDetailsMoveButton = ({ action, moving }: TaskDetailsMoveButtonProps): JSX.Element => (
  <Button type='button' variant='secondary' disabled={moving} onClick={action.onClick}>
    Move to {action.label}
  </Button>
);

export const TaskDetailsDialog = ({
  task,
  availableLabels,
  createdAtLabel,
  moveActions,
  augmentation,
  form,
  status,
  actions
}: TaskDetailsDialogProps): JSX.Element => (
  <Dialog open={Boolean(task)} onOpenChange={actions.onTaskDetailsOpenChange}>
    <Dialog.Portal>
      <Dialog.Backdrop />
      <Dialog.Popup class='flex max-h-[90vh] w-[min(92vw,720px)] flex-col gap-5 overflow-hidden'>
        {task && (
          <>
            <div class='flex items-start justify-between gap-4'>
              <div class='flex flex-col gap-2'>
                <Dialog.Title>{formatTaskDisplayId(task)}</Dialog.Title>
                <Dialog.Description>Edit ticket details and provider labels.</Dialog.Description>
              </div>
              <Dialog.Close>
                <Button type='button' variant='ghost'>
                  Close
                </Button>
              </Dialog.Close>
            </div>
            <div class='grid gap-3 text-sm text-text-secondary md:grid-cols-2'>
              <span>Status: {task.status}</span>
              <span>Priority: {task.priority}</span>
              <span>Assignee: {task.assigneeName ?? 'Unassigned'}</span>
              <span>Created: {createdAtLabel}</span>
            </div>
            <TaskUsageSummary augmentation={augmentation} variant='dialog' />
            <form
              class='flex min-h-0 flex-1 flex-col gap-4 overflow-auto'
              onSubmit={actions.onSubmitTaskEdit}
            >
              <div class='flex flex-col gap-2'>
                <Label for='task-details-title'>Title</Label>
                <Input
                  id='task-details-title'
                  value={form.editTitle}
                  onInput={actions.onEditTaskTitleInput}
                  disabled={status.updatingTask}
                  invalid={Boolean(form.editError)}
                />
              </div>
              <div class='flex min-h-44 flex-col gap-2'>
                <Label for='task-details-body'>Body</Label>
                <TextArea
                  id='task-details-body'
                  value={form.editBody}
                  onInput={actions.onEditTaskBodyInput}
                  disabled={status.updatingTask}
                  class='min-h-44 flex-1 resize-y leading-6'
                />
              </div>
              {form.editError && <p class='text-sm text-error'>{form.editError}</p>}
              <div class='flex flex-col gap-3 border border-border-base bg-bg-input p-3'>
                <div class='flex items-center justify-between gap-3'>
                  <p class='text-xs uppercase tracking-wider text-text-muted'>Labels</p>
                  {status.updatingTaskLabel && (
                    <span class='text-[10px] uppercase tracking-wider text-text-muted'>
                      Saving…
                    </span>
                  )}
                </div>
                {availableLabels.length > 0 ? (
                  <div class='grid gap-2 sm:grid-cols-2'>
                    {availableLabels.map((label) => {
                      const checked = form.editLabelIds.includes(label.id);
                      const checkboxId = `task-label-${label.id}`;

                      return (
                        <div
                          key={label.id}
                          class='flex items-center gap-2 border border-border-base bg-bg-card p-2 text-xs text-text-secondary transition-colors hover:border-border-focus hover:text-text-primary'
                        >
                          <Checkbox
                            id={checkboxId}
                            checked={checked}
                            disabled={status.updatingTaskLabel}
                            onCheckedChange={(nextChecked) =>
                              actions.onToggleTaskLabel(label.id, nextChecked)
                            }
                          />
                          <label
                            for={checkboxId}
                            class='flex min-w-0 flex-1 cursor-pointer items-center gap-2'
                          >
                            <span
                              class='size-2 shrink-0 border border-border-base'
                              style={{ background: label.color }}
                            />
                            <span class='truncate'>{label.name}</span>
                          </label>
                          <Button
                            type='button'
                            variant='ghost'
                            disabled={status.updatingTaskLabel}
                            aria-label={`Delete label ${label.name}`}
                            class='shrink-0 px-2 py-1 text-[10px]'
                            onClick={() => actions.onDeleteLabel(label.id)}
                          >
                            Delete
                          </Button>
                        </div>
                      );
                    })}
                  </div>
                ) : (
                  <p class='text-xs text-text-muted'>
                    No provider labels configured for this board.
                  </p>
                )}
              </div>
              <div class='flex flex-wrap justify-between gap-2'>
                <div class='flex flex-wrap gap-2'>
                  {moveActions.map((action) => (
                    <TaskDetailsMoveButton
                      key={action.value}
                      action={action}
                      moving={status.moving}
                    />
                  ))}
                </div>
                <Button type='submit' variant='primary' disabled={status.updatingTask}>
                  {status.updatingTask ? 'Saving…' : 'Save changes'}
                </Button>
              </div>
            </form>
          </>
        )}
      </Dialog.Popup>
    </Dialog.Portal>
  </Dialog>
);
