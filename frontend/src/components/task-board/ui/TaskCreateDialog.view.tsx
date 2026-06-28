import type { JSX } from 'preact';
import type { SelectOption } from '../../../types/shared';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
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
    'onBodyInput' | 'onCreateDialogOpenChange' | 'onStatusChange' | 'onSubmitTask' | 'onTitleInput'
  >;
}

export const TaskCreateDialog = ({
  open,
  form,
  status,
  statusOptions,
  actions
}: TaskCreateDialogProps): JSX.Element => (
  <Dialog open={open} onOpenChange={actions.onCreateDialogOpenChange}>
    <Dialog.Portal>
      <Dialog.Backdrop />
      <Dialog.Popup class='flex w-[min(92vw,720px)] flex-col gap-5'>
        <div class='flex items-start justify-between gap-4'>
          <div>
            <Dialog.Title>Create task</Dialog.Title>
            <Dialog.Description>Add a provider ticket through Vulcanum.</Dialog.Description>
          </div>
          <Dialog.Close>
            <Button type='button' variant='ghost'>
              Close
            </Button>
          </Dialog.Close>
        </div>
        <form onSubmit={actions.onSubmitTask} class='flex flex-col gap-4'>
          {form.createError && <ErrorBanner message={form.createError} />}
          <div class='flex flex-col gap-2'>
            <Label for='task-create-title'>Title</Label>
            <Input
              id='task-create-title'
              value={form.title}
              onInput={actions.onTitleInput}
              disabled={status.creating}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='task-create-description'>Description</Label>
            <TextArea
              id='task-create-description'
              value={form.body}
              onInput={actions.onBodyInput}
              rows={5}
              disabled={status.creating}
            />
          </div>
          <div class='flex flex-col gap-2'>
            <Label for='task-create-status'>Status</Label>
            <Select
              id='task-create-status'
              value={form.status}
              onValueChange={actions.onStatusChange}
              items={statusOptions}
              disabled={status.creating}
            />
          </div>
          <div class='flex justify-end gap-2'>
            <Button type='submit' variant='primary' disabled={status.creating}>
              {status.creating ? 'Creating…' : 'Create task'}
            </Button>
          </div>
        </form>
      </Dialog.Popup>
    </Dialog.Portal>
  </Dialog>
);
