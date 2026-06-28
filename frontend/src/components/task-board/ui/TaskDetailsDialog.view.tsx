import type { JSX } from 'preact';
import type { TaskBoardTask } from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import type { TaskBoardActions, TaskBoardMoveAction, TaskBoardStatusState } from '../types';

interface TaskDetailsMoveButtonProps {
  action: TaskBoardMoveAction;
  moving: boolean;
}

interface TaskDetailsDialogProps {
  task: TaskBoardTask | null;
  createdAtLabel: string | null;
  moveActions: TaskBoardMoveAction[];
  status: Pick<TaskBoardStatusState, 'moving'>;
  actions: Pick<TaskBoardActions, 'onTaskDetailsOpenChange'>;
}

const TaskDetailsMoveButton = ({ action, moving }: TaskDetailsMoveButtonProps): JSX.Element => (
  <Button type='button' variant='secondary' disabled={moving} onClick={action.onClick}>
    Move to {action.label}
  </Button>
);

export const TaskDetailsDialog = ({
  task,
  createdAtLabel,
  moveActions,
  status,
  actions
}: TaskDetailsDialogProps): JSX.Element => (
  <Dialog open={Boolean(task)} onOpenChange={actions.onTaskDetailsOpenChange}>
    <Dialog.Portal>
      <Dialog.Backdrop />
      <Dialog.Popup class='flex max-h-[90vh] w-[min(92vw,640px)] flex-col gap-5 overflow-hidden'>
        {task && (
          <>
            <div class='flex items-start justify-between gap-4'>
              <div class='flex flex-col gap-2'>
                <Dialog.Title>{task.title}</Dialog.Title>
                <Dialog.Description>{task.number ? `#${task.number}` : task.id}</Dialog.Description>
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
            <div class='min-h-0 flex-1 overflow-auto border border-border-base bg-bg-input p-4 text-sm leading-6 text-text-secondary'>
              {task.description ? (
                <p class='whitespace-pre-wrap'>{task.description}</p>
              ) : (
                <p class='text-text-muted'>No task body.</p>
              )}
            </div>
            <div class='flex flex-wrap gap-2'>
              {moveActions.map((action) => (
                <TaskDetailsMoveButton key={action.value} action={action} moving={status.moving} />
              ))}
            </div>
          </>
        )}
      </Dialog.Popup>
    </Dialog.Portal>
  </Dialog>
);
