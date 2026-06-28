import type { JSX } from 'preact';
import { Checkbox } from '../../shared/ui/Checkbox.view';
import { Input } from '../../shared/ui/Input.view';
import type { TaskBoardRepositoryItem, TaskBoardRepositorySettingsData } from '../types';
import { TaskBoardSettingsSection } from './TaskBoardSettingsSection.view';

interface RepoCheckboxProps {
  repo: TaskBoardRepositoryItem;
  disabled: boolean;
}

interface TaskBoardRepositorySettingsProps {
  data: TaskBoardRepositorySettingsData;
  loading: boolean;
  disabled: boolean;
  onFilterRepos: (event: Event) => void;
}

const RepoCheckbox = ({ repo, disabled }: RepoCheckboxProps): JSX.Element => (
  <label for={`repo-${repo.value}`} class='flex items-center gap-2 text-sm text-text-secondary'>
    <Checkbox
      id={`repo-${repo.value}`}
      checked={repo.checked}
      disabled={disabled}
      onCheckedChange={repo.onToggle}
    />
    <span>{repo.label}</span>
  </label>
);

export const TaskBoardRepositorySettings = ({
  data,
  loading,
  disabled,
  onFilterRepos
}: TaskBoardRepositorySettingsProps): JSX.Element => (
  <TaskBoardSettingsSection
    title='Repositories'
    description='Pinned repositories are passed to workers for this project.'
    hasOverrides={data.hasOverrides}
  >
    {loading && <p class='text-xs text-text-muted'>Loading repositories…</p>}
    {!loading && !data.hasRepos && (
      <p class='text-xs text-text-muted'>No GitHub repositories are available.</p>
    )}
    {data.hasRepos && (
      <div class='flex flex-col gap-4'>
        <Input
          aria-label='Filter repositories'
          placeholder='Type to filter repositories'
          value={data.filter}
          disabled={disabled}
          onInput={onFilterRepos}
        />
        {data.hasSelectedRepos && (
          <section class='flex flex-col gap-2'>
            <h4 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
              Selected repositories
            </h4>
            <div class='grid gap-2 border border-accent bg-bg-input p-3 md:grid-cols-2'>
              {data.selectedRepos.map((repo) => (
                <RepoCheckbox key={repo.value} repo={repo} disabled={disabled} />
              ))}
            </div>
          </section>
        )}
        <section class='flex flex-col gap-2'>
          <h4 class='text-xs font-medium uppercase tracking-wider text-text-muted'>
            Available repositories
          </h4>
          {data.hasFilteredRepos ? (
            <div class='grid max-h-72 gap-2 overflow-auto border border-border-base bg-bg-input p-3 md:grid-cols-2'>
              {data.filteredRepos.map((repo) => (
                <RepoCheckbox key={repo.value} repo={repo} disabled={disabled} />
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
