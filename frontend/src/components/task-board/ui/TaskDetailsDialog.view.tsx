import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardTask } from '../../../types/task-board';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import type { TaskBoardActions, TaskBoardStatusState } from '../types';

interface TaskDetailsMoveButtonProps {
  taskId: string;
  status: SelectOption;
  moving: boolean;
  onMoveTask: (taskId: string, status: string) => void;
}

interface TaskDetailsDialogProps {
  task: TaskBoardTask | null;
  statusOptions: SelectOption[];
  status: Pick<TaskBoardStatusState, 'moving'>;
  actions: Pick<TaskBoardActions, 'onCloseTask' | 'onMoveTask'>;
}

const TaskDetailsMoveButton = ({
  taskId,
  status,
  moving,
  onMoveTask
}: TaskDetailsMoveButtonProps): JSX.Element => {
  const moveTask = useCallback(() => {
    onMoveTask(taskId, status.value);
  }, [onMoveTask, status.value, taskId]);

  return (
    <Button type='button' variant='secondary' disabled={moving} onClick={moveTask}>
      Move to {status.label}
    </Button>
  );
};

export const TaskDetailsDialog = ({
  task,
  statusOptions,
  status,
  actions
}: TaskDetailsDialogProps): JSX.Element => {
  const closeWhenDismissed = useCallback(
    (nextOpen: boolean) => {
      if (nextOpen) return;

      actions.onCloseTask();
    },
    [actions]
  );

  return (
    <Dialog open={Boolean(task)} onOpenChange={closeWhenDismissed}>
      <Dialog.Portal>
        <Dialog.Backdrop />
        <Dialog.Popup class='flex max-h-[90vh] w-[min(92vw,640px)] flex-col gap-5 overflow-hidden'>
          {task && (
            <>
              <div class='flex items-start justify-between gap-4'>
                <div class='flex flex-col gap-2'>
                  <Dialog.Title>{task.title}</Dialog.Title>
                  <Dialog.Description>
                    {task.number ? `#${task.number}` : task.id}
                  </Dialog.Description>
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
                <span>Created: {new Date(task.createdAt).toLocaleString()}</span>
              </div>
              <div class='min-h-0 flex-1 overflow-auto border border-border-base bg-bg-input p-4 text-sm leading-6 text-text-secondary'>
                {task.description ? (
                  <p class='whitespace-pre-wrap'>{task.description}</p>
                ) : (
                  <p class='text-text-muted'>No task body.</p>
                )}
              </div>
              <div class='flex flex-wrap gap-2'>
                {statusOptions
                  .filter((option) => option.value !== task.status)
                  .map((option) => (
                    <TaskDetailsMoveButton
                      key={option.value}
                      taskId={task.id}
                      status={option}
                      moving={status.moving}
                      onMoveTask={actions.onMoveTask}
                    />
                  ))}
              </div>
            </>
          )}
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
};
