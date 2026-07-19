import type { JSX } from 'preact';
import type { TaskBoardProjectUsage, TaskBoardUsageCounters } from '../../../types/task-board';
import { formatTokenCount } from '../../../utils/format';
import { RunUsageStats } from '../../runs/ui/runs-table/RunUsageStats';

interface ProjectUsageSummaryProps {
  usage: TaskBoardProjectUsage;
}

interface UsagePeriodProps {
  label: string;
  detail?: string;
  counters: TaskBoardUsageCounters;
  emptyMessage: string;
  class?: string;
}

const hasUsage = (counters: TaskBoardUsageCounters): boolean =>
  counters.tokensUsed !== 0 || counters.finishedRunsCount !== 0;

const finishedRunsLabel = (count: number): string =>
  `${formatTokenCount(count)} finished ${count === 1 ? 'run' : 'runs'}`;

const UsagePeriod = ({
  label,
  detail,
  counters,
  emptyMessage,
  class: className
}: UsagePeriodProps): JSX.Element => (
  <div class={`flex min-w-0 flex-col gap-3 p-4 ${className ?? ''}`}>
    <div class='flex flex-col gap-1'>
      <span class='text-xs font-medium uppercase tracking-wider text-text-muted'>{label}</span>
      {detail && <span class='text-xs text-text-muted'>{detail}</span>}
    </div>
    {hasUsage(counters) ? (
      <>
        <div class='flex flex-wrap items-baseline gap-x-3 gap-y-1'>
          <span class='font-mono text-xl font-semibold tabular-nums text-text-primary'>
            {formatTokenCount(counters.tokensUsed)} tokens
          </span>
          <span class='font-mono text-xs tabular-nums text-text-muted'>
            {finishedRunsLabel(counters.finishedRunsCount)}
          </span>
        </div>
        <div role='group' aria-label={`${label} token breakdown`}>
          <RunUsageStats run={counters} />
        </div>
      </>
    ) : (
      <p class='text-sm text-text-muted'>{emptyMessage}</p>
    )}
  </div>
);

export const ProjectUsageSummary = ({ usage }: ProjectUsageSummaryProps): JSX.Element => {
  const projectHasUsage = hasUsage(usage.total);

  return (
    <section aria-label='Project usage' class='border border-border-base bg-bg-panel'>
      <div class='flex flex-col gap-1 border-b border-border-base px-4 py-3'>
        <h3 class='text-sm font-medium text-text-primary'>Project usage</h3>
        <p class='text-xs text-text-muted'>Accepted work run token usage</p>
      </div>
      {projectHasUsage ? (
        <div class='grid md:grid-cols-2'>
          <UsagePeriod
            label='Total'
            counters={usage.total}
            emptyMessage='No accepted run usage recorded for this project yet.'
          />
          <UsagePeriod
            label='This week'
            detail='Monday 00:00 UTC to now'
            counters={usage.thisWeek}
            emptyMessage='No usage this week.'
            class='border-t border-border-base md:border-t-0 md:border-l'
          />
        </div>
      ) : (
        <p class='p-4 text-sm text-text-muted'>
          No accepted run usage recorded for this project yet.
        </p>
      )}
    </section>
  );
};
