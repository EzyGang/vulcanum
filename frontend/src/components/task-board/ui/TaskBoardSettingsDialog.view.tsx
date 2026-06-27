import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import type {
  TaskBoardActions,
  TaskBoardColumnRoles,
  TaskBoardFormState,
  TaskBoardStatusState
} from '../types';
import { TaskBoardColumnSettings } from './TaskBoardColumnSettings.view';
import { TaskBoardProjectSettings } from './TaskBoardProjectSettings.view';
import { TaskBoardRepositorySettings } from './TaskBoardRepositorySettings.view';
import { TaskBoardReviewSettings } from './TaskBoardReviewSettings.view';

interface TaskBoardSettingsDialogProps {
  open: boolean;
  form: TaskBoardFormState['settings'];
  repoItems: { value: string; label: string }[];
  columnRoles: TaskBoardColumnRoles;
  selectedRepoNames: string[];
  statusOptions: { value: string; label: string }[];
  status: Pick<
    TaskBoardStatusState,
    'connected' | 'connectingRepos' | 'reposLoading' | 'savingSettings'
  >;
  actions: Pick<
    TaskBoardActions,
    | 'onCloseSettings'
    | 'onToggleRepo'
    | 'onSettingsPromptInput'
    | 'onSettingsAgentsInput'
    | 'onSettingsReviewEnabledChange'
    | 'onSettingsReviewPickupColumnChange'
    | 'onSettingsReviewMaxTurnsInput'
    | 'onSettingsReviewPromptInput'
    | 'onSettingsMaxInProgressInput'
    | 'onSetColumnRole'
    | 'onSubmitSettings'
  >;
}

export const TaskBoardSettingsDialog = ({
  open,
  form,
  repoItems,
  selectedRepoNames,
  columnRoles,
  statusOptions,
  status,
  actions
}: TaskBoardSettingsDialogProps): JSX.Element => {
  const closeWhenDismissed = useCallback(
    (nextOpen: boolean) => {
      if (nextOpen) return;

      actions.onCloseSettings();
    },
    [actions]
  );
  const settingsDisabled = status.savingSettings || !status.connected;
  const repoControlsDisabled = status.connectingRepos || !status.connected;

  return (
    <Dialog open={open} onOpenChange={closeWhenDismissed}>
      <Dialog.Portal>
        <Dialog.Backdrop />
        <Dialog.Popup class='w-[min(92vw,760px)] max-h-[90vh] overflow-auto'>
          <form onSubmit={actions.onSubmitSettings} class='flex flex-col gap-5'>
            <div class='flex items-start justify-between gap-4'>
              <div class='flex flex-col gap-2'>
                <Dialog.Title>Board settings</Dialog.Title>
                <Dialog.Description>
                  Pin repositories and override task automation for this provider project.
                </Dialog.Description>
              </div>
              <Dialog.Close>
                <Button type='button' variant='ghost'>
                  Close
                </Button>
              </Dialog.Close>
            </div>

            <span class='text-xs uppercase tracking-wider text-text-muted'>
              {status.connected
                ? 'Project configuration connected'
                : 'Project configuration missing'}
            </span>
            {!status.connected && (
              <p class='border border-warning-border bg-warning-bg p-3 text-xs text-warning'>
                Add this provider project to Vulcanum before editing board settings.
              </p>
            )}

            <TaskBoardRepositorySettings
              repoItems={repoItems}
              selectedRepoNames={selectedRepoNames}
              loading={status.reposLoading}
              disabled={repoControlsDisabled}
              onToggleRepo={actions.onToggleRepo}
            />
            <TaskBoardColumnSettings
              columnRoles={columnRoles}
              statusOptions={statusOptions}
              disabled={settingsDisabled}
              actions={actions}
            />
            <TaskBoardProjectSettings form={form} disabled={settingsDisabled} actions={actions} />
            <TaskBoardReviewSettings
              form={form}
              statusOptions={statusOptions}
              disabled={settingsDisabled}
              actions={actions}
            />

            <div class='flex justify-end gap-2'>
              <Button type='submit' variant='primary' disabled={settingsDisabled}>
                {status.savingSettings ? 'Saving…' : 'Save settings'}
              </Button>
            </div>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
};
