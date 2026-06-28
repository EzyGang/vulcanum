import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import type {
  TaskBoardActions,
  TaskBoardFormState,
  TaskBoardProjectSettingsData,
  TaskBoardRepositorySettingsData,
  TaskBoardReviewSettingsData,
  TaskBoardStatusState
} from '../types';
import { TaskBoardProjectSettings } from './TaskBoardProjectSettings.view';
import { TaskBoardRepositorySettings } from './TaskBoardRepositorySettings.view';
import { TaskBoardReviewSettings } from './TaskBoardReviewSettings.view';

interface TaskBoardSettingsDialogProps {
  open: boolean;
  form: TaskBoardFormState['settings'];
  repositorySettings: TaskBoardRepositorySettingsData;
  projectSettings: TaskBoardProjectSettingsData;
  reviewSettings: TaskBoardReviewSettingsData;
  status: Pick<
    TaskBoardStatusState,
    'connected' | 'reposLoading' | 'savingSettings' | 'settingsDisabled' | 'repoControlsDisabled'
  >;
  actions: Pick<
    TaskBoardActions,
    | 'onFilterRepos'
    | 'onSettingsDialogOpenChange'
    | 'onSettingsPromptInput'
    | 'onSettingsAgentsInput'
    | 'onSettingsReviewEnabledChange'
    | 'onSettingsReviewMaxTurnsInput'
    | 'onSettingsReviewPromptInput'
    | 'onSettingsMaxInProgressInput'
    | 'onSubmitSettings'
  >;
}

export const TaskBoardSettingsDialog = ({
  open,
  form,
  repositorySettings,
  projectSettings,
  reviewSettings,
  status,
  actions
}: TaskBoardSettingsDialogProps): JSX.Element => (
  <Dialog open={open} onOpenChange={actions.onSettingsDialogOpenChange}>
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
            {status.connected ? 'Project configuration connected' : 'Project configuration missing'}
          </span>
          {!status.connected && (
            <p class='border border-warning-border bg-warning-bg p-3 text-xs text-warning'>
              Add this provider project to Vulcanum before editing board settings.
            </p>
          )}

          <TaskBoardRepositorySettings
            data={repositorySettings}
            loading={status.reposLoading}
            disabled={status.repoControlsDisabled}
            onFilterRepos={actions.onFilterRepos}
          />
          <TaskBoardProjectSettings
            form={form}
            data={projectSettings}
            disabled={status.settingsDisabled}
            actions={actions}
          />
          <TaskBoardReviewSettings
            form={form}
            data={reviewSettings}
            disabled={status.settingsDisabled}
            actions={actions}
          />

          <div class='flex justify-end gap-2'>
            <Button type='submit' variant='primary' disabled={status.settingsDisabled}>
              {status.savingSettings ? 'Saving…' : 'Save settings'}
            </Button>
          </div>
        </form>
      </Dialog.Popup>
    </Dialog.Portal>
  </Dialog>
);
