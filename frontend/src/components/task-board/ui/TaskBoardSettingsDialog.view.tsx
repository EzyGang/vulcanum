import type { JSX } from 'preact';
import { useCallback } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Button } from '../../shared/ui/Button.view';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import type { TaskBoardActions, TaskBoardStatusState } from '../types';

interface RepoCheckboxProps {
  repo: SelectOption;
  checked: boolean;
  disabled: boolean;
  onToggleRepo: (repoFullName: string) => void;
}

interface TaskBoardSettingsDialogProps {
  open: boolean;
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  status: Pick<TaskBoardStatusState, 'connected' | 'connectingRepos' | 'reposLoading'>;
  actions: Pick<TaskBoardActions, 'onCloseSettings' | 'onToggleRepo'>;
}

const RepoCheckbox = ({
  repo,
  checked,
  disabled,
  onToggleRepo
}: RepoCheckboxProps): JSX.Element => {
  const toggleRepo = useCallback(() => {
    onToggleRepo(repo.value);
  }, [onToggleRepo, repo.value]);

  return (
    <label for={`repo-${repo.value}`} class='flex items-center gap-2 text-sm text-text-secondary'>
      <Checkbox
        id={`repo-${repo.value}`}
        checked={checked}
        disabled={disabled}
        onCheckedChange={toggleRepo}
      />
      <span>{repo.label}</span>
    </label>
  );
};

export const TaskBoardSettingsDialog = ({
  open,
  repoItems,
  selectedRepoNames,
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

  return (
    <Dialog open={open} onOpenChange={closeWhenDismissed}>
      <Dialog.Portal>
        <Dialog.Backdrop />
        <Dialog.Popup class='flex w-[min(92vw,720px)] flex-col gap-5'>
          <div class='flex items-start justify-between gap-4'>
            <div class='flex flex-col gap-2'>
              <Dialog.Title>Board settings</Dialog.Title>
              <Dialog.Description>
                Assign GitHub repositories related to this provider project.
              </Dialog.Description>
            </div>
            <Dialog.Close>
              <Button type='button' variant='ghost'>
                Close
              </Button>
            </Dialog.Close>
          </div>
          <span class='text-xs uppercase tracking-wider text-text-muted'>
            {status.connected ? 'Connected' : 'Not connected'}
          </span>
          {status.reposLoading && <p class='text-xs text-text-muted'>Loading repositories…</p>}
          {!status.reposLoading && repoItems.length === 0 && (
            <p class='text-xs text-text-muted'>No GitHub repositories are available.</p>
          )}
          {repoItems.length > 0 && (
            <div class='grid max-h-72 gap-2 overflow-auto border border-border-base bg-bg-input p-3 md:grid-cols-2'>
              {repoItems.map((repo) => (
                <RepoCheckbox
                  key={repo.value}
                  repo={repo}
                  checked={selectedRepoNames.includes(repo.value)}
                  disabled={status.connectingRepos}
                  onToggleRepo={actions.onToggleRepo}
                />
              ))}
            </div>
          )}
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
};
