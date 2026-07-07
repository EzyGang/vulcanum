import type { JSX } from 'preact';
import type { TaskBoardTaskAugmentation } from '../../../types/task-board';
import { formatTokenCount } from '../../../utils/format';
import { hasRunUsageStats, RunUsageStats } from '../../runs/ui/runs-table/RunUsageStats';

interface TaskUsageSummaryProps {
  augmentation: TaskBoardTaskAugmentation | null;
  variant: 'card' | 'dialog';
}

const FinishedRunsLabel = ({ count }: { count: number }): JSX.Element => (
  <span class='text-[10px] uppercase tracking-wider text-text-muted'>
    {count} finished {count === 1 ? 'run' : 'runs'}
  </span>
);

export const TaskUsageSummary = ({
  augmentation,
  variant
}: TaskUsageSummaryProps): JSX.Element | null => {
  if (!augmentation && variant === 'card') {
    return null;
  }

  if (!augmentation) {
    return (
      <section class='flex flex-col gap-3 border border-border-base bg-bg-input p-3'>
        <p class='text-xs uppercase tracking-wider text-text-muted'>Cumulative usage</p>
        <p class='text-xs text-text-muted'>No usage recorded yet.</p>
      </section>
    );
  }

  const content = hasRunUsageStats(augmentation) ? (
    <RunUsageStats run={augmentation} />
  ) : (
    <span class='font-mono text-xs tabular-nums text-text-secondary'>
      {formatTokenCount(augmentation.tokensUsed)} tokens
    </span>
  );

  if (variant === 'card') {
    return (
      <div role='group' aria-label='Cumulative usage' class='flex flex-col gap-1'>
        <div class='flex items-center justify-between gap-2'>
          <span class='text-[10px] uppercase tracking-wider text-text-muted'>Usage</span>
          <FinishedRunsLabel count={augmentation.finishedRunsCount} />
        </div>
        {content}
      </div>
    );
  }

  return (
    <section class='flex flex-col gap-3 border border-border-base bg-bg-input p-3'>
      <div class='flex items-center justify-between gap-3'>
        <p class='text-xs uppercase tracking-wider text-text-muted'>Cumulative usage</p>
        <FinishedRunsLabel count={augmentation.finishedRunsCount} />
      </div>
      {content}
    </section>
  );
};
