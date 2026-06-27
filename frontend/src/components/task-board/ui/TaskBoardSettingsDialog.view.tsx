import { useSignal } from '@preact/signals';
import type { JSX } from 'preact';
import { useCallback, useMemo } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Button } from '../../shared/ui/Button.view';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { Input } from '../../shared/ui/Input.view';
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

  const repoFilter = useSignal('');
  const selectedRepoNameSet = useMemo(() => new Set(selectedRepoNames), [selectedRepoNames]);
  const selectedRepos = useMemo(
    () =>
      selectedRepoNames.map(
        (repoFullName) =>
          repoItems.find((repo) => repo.value === repoFullName) ?? {
            value: repoFullName,
            label: repoFullName
          }
      ),
    [repoItems, selectedRepoNames]
  );
  const normalizedRepoFilter = repoFilter.value.trim().toLocaleLowerCase();
  const filteredRepoItems = repoItems.filter(
    (repo) =>
      !selectedRepoNameSet.has(repo.value) &&
      (normalizedRepoFilter.length === 0 ||
        repo.label.toLocaleLowerCase().includes(normalizedRepoFilter) ||
        repo.value.toLocaleLowerCase().includes(normalizedRepoFilter))
  );

  const filterRepos = useCallback(
    (event: Event) => {
      repoFilter.value = (event.target as HTMLInputElement).value;
    },
    [repoFilter]
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
            <div class='flex flex-col gap-4'>
              <Input
                aria-label='Filter repositories'
                placeholder='Filter repositories'
                value={repoFilter.value}
                disabled={status.connectingRepos}
                onInput={filterRepos}
              />
              {selectedRepos.length > 0 && (
                <section class='flex flex-col gap-2'>
                  <h3 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
                    Selected repositories
                  </h3>
                  <div class='grid gap-2 border border-border-accent bg-accent-muted/10 p-3 md:grid-cols-2'>
                    {selectedRepos.map((repo) => (
                      <RepoCheckbox
                        key={repo.value}
                        repo={repo}
                        checked={true}
                        disabled={status.connectingRepos}
                        onToggleRepo={actions.onToggleRepo}
                      />
                    ))}
                  </div>
                </section>
              )}
              <section class='flex flex-col gap-2'>
                <h3 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
                  Available repositories
                </h3>
                {filteredRepoItems.length > 0 ? (
                  <div class='grid max-h-72 gap-2 overflow-auto border border-border-base bg-bg-input p-3 md:grid-cols-2'>
                    {filteredRepoItems.map((repo) => (
                      <RepoCheckbox
                        key={repo.value}
                        repo={repo}
                        checked={false}
                        disabled={status.connectingRepos}
                        onToggleRepo={actions.onToggleRepo}
                      />
                    ))}
                  </div>
                ) : (
                  <p class='border border-border-base bg-bg-input p-3 text-xs text-text-muted'>
                    No repositories match this filter.
                  </p>
                )}
              </section>
            </div>
          )}
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
};
