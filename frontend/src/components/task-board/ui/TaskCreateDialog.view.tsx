import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Select } from '../../shared/ui/Select.view';
import { TextArea } from '../../shared/ui/TextArea.view';
import type { TaskBoardActions, TaskBoardFormState, TaskBoardStatusState } from '../types';

interface TaskCreateDialogProps {
  open: boolean;
  form: TaskBoardFormState;
  status: Pick<TaskBoardStatusState, 'creating'>;
  statusOptions: SelectOption[];
  actions: Pick<
    TaskBoardActions,
    'onBodyInput' | 'onCloseCreateTask' | 'onStatusChange' | 'onSubmitTask' | 'onTitleInput'
  >;
}

export const TaskCreateDialog = ({
  open,
  form,
  status,
  statusOptions,
  actions
}: TaskCreateDialogProps): JSX.Element => {
  const closeWhenDismissed = useCallback(
    (nextOpen: boolean) => {
      if (nextOpen) return;

      actions.onCloseCreateTask();
    },
    [actions]
  );

  return (
    <Dialog open={open} onOpenChange={closeWhenDismissed}>
      <Dialog.Portal>
        <Dialog.Backdrop />
        <Dialog.Popup class='flex w-[min(92vw,720px)] flex-col gap-5'>
          <div class='flex items-start justify-between gap-4'>
            <div class='flex flex-col gap-2'>
              <Dialog.Title>Create task</Dialog.Title>
              <Dialog.Description>
                Add a provider task without leaving the board.
              </Dialog.Description>
            </div>
            <Dialog.Close>
              <Button type='button' variant='ghost'>
                Close
              </Button>
            </Dialog.Close>
          </div>
          <form onSubmit={actions.onSubmitTask} class='flex flex-col gap-4'>
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
                rows={6}
                placeholder='Task details for whoever picks this up'
              />
            </div>
            <div class='flex flex-col gap-2'>
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
            </div>
            {form.createError && <ErrorBanner message={form.createError} />}
            <Button type='submit' variant='primary' disabled={status.creating}>
              {status.creating ? 'Creating…' : 'Create task'}
            </Button>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
};
