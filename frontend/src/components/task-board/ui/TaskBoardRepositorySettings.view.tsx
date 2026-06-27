import { useSignal } from '@preact/signals';
import type { JSX } from 'preact';
import { useCallback, useMemo } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Input } from '../../shared/ui/Input.view';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface RepoCheckboxProps {
  repo: SelectOption;
  checked: boolean;
  disabled: boolean;
  onToggleRepo: (repoFullName: string) => void;
}

interface TaskBoardRepositorySettingsProps {
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  loading: boolean;
  disabled: boolean;
  onToggleRepo: (repoFullName: string) => void;
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

export const TaskBoardRepositorySettings = ({
  repoItems,
  selectedRepoNames,
  loading,
  disabled,
  onToggleRepo
}: TaskBoardRepositorySettingsProps): JSX.Element => {
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
    <TaskBoardSettingsSection
      title='Repositories'
      description='Pinned repositories are passed to workers for this project.'
    >
      {loading && <p class='text-xs text-text-muted'>Loading repositories…</p>}
      {!loading && repoItems.length === 0 && (
        <p class='text-xs text-text-muted'>No GitHub repositories are available.</p>
      )}
      {repoItems.length > 0 && (
        <div class='flex flex-col gap-4'>
          <Input
            aria-label='Filter repositories'
            placeholder='Type to filter repositories'
            value={repoFilter.value}
            disabled={disabled}
            onInput={filterRepos}
          />
          {selectedRepos.length > 0 && (
            <section class='flex flex-col gap-2'>
              <h4 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
                Selected repositories
              </h4>
              <div class='grid gap-2 border border-accent bg-bg-input p-3 md:grid-cols-2'>
                {selectedRepos.map((repo) => (
                  <RepoCheckbox
                    key={repo.value}
                    repo={repo}
                    checked={true}
                    disabled={disabled}
                    onToggleRepo={onToggleRepo}
                  />
                ))}
              </div>
            </section>
          )}
          <section class='flex flex-col gap-2'>
            <h4 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
              Available repositories
            </h4>
            {filteredRepoItems.length > 0 ? (
              <div class='grid max-h-72 gap-2 overflow-auto border border-border-base bg-bg-input p-3 md:grid-cols-2'>
                {filteredRepoItems.map((repo) => (
                  <RepoCheckbox
                    key={repo.value}
                    repo={repo}
                    checked={false}
                    disabled={disabled}
                    onToggleRepo={onToggleRepo}
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
    </TaskBoardSettingsSection>
  );
};
