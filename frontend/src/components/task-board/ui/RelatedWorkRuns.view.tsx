import type { JSX } from 'preact';
import { WORK_RUN_TYPE_LABELS } from '../../../types/runs';
import type { TaskBoardRelatedWorkRun } from '../../../types/task-board';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { RelatedWorkRunUsage } from './RelatedWorkRunUsage';

interface RelatedWorkRunsProps {
  runs: TaskBoardRelatedWorkRun[];
  variant: 'card' | 'dialog';
}

const MAX_RELATED_RUNS = 3;

const RelatedRunRow = ({ run, compact }: { run: TaskBoardRelatedWorkRun; compact: boolean }) => (
  <li class='flex flex-col gap-2 border border-border-base bg-bg-card p-2'>
    <div class='flex flex-wrap items-center gap-2'>
      <span class='border border-border-base bg-bg-panel px-2 py-0.5 font-mono text-[10px] uppercase tracking-wider text-text-secondary'>
        {WORK_RUN_TYPE_LABELS[run.workType]}
      </span>
      <StatusBadge status={run.status} />
    </div>
    <div class={compact ? 'max-w-full overflow-hidden' : ''}>
      <RelatedWorkRunUsage run={run} />
    </div>
  </li>
);

export const RelatedWorkRuns = ({ runs, variant }: RelatedWorkRunsProps): JSX.Element | null => {
  const visibleRuns = runs.slice(0, MAX_RELATED_RUNS);

  if (visibleRuns.length === 0 && variant === 'card') {
    return null;
  }

  if (variant === 'card') {
    return (
      <ul aria-label='Related work runs' class='flex flex-col gap-1'>
        {visibleRuns.map((run) => (
          <RelatedRunRow key={run.id} run={run} compact />
        ))}
      </ul>
    );
  }

  return (
    <section class='flex flex-col gap-3 border border-border-base bg-bg-input p-3'>
      <div class='flex items-center justify-between gap-3'>
        <p class='text-xs uppercase tracking-wider text-text-muted'>Related runs</p>
        {visibleRuns.length > 0 && (
          <span class='text-[10px] uppercase tracking-wider text-text-muted'>Latest 3</span>
        )}
      </div>
      {visibleRuns.length > 0 ? (
        <ul class='grid gap-2'>
          {visibleRuns.map((run) => (
            <RelatedRunRow key={run.id} run={run} compact={false} />
          ))}
        </ul>
      ) : (
        <p class='text-xs text-text-muted'>No related work runs yet.</p>
      )}
    </section>
  );
};
