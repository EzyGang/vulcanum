import type { TaskBoardProjectUsage, TaskBoardUsageCounters } from '../../../types/task-board';
import { formatTokenCount } from '../../../utils/format';
import type { TaskBoardProjectUsagePeriodData, TaskBoardProjectUsageSummaryData } from '../types';

const hasUsage = (counters: TaskBoardUsageCounters): boolean =>
  counters.tokensUsed !== 0 || counters.finishedRunsCount !== 0;

const finishedRunsLabel = (count: number): string =>
  `${formatTokenCount(count)} finished ${count === 1 ? 'run' : 'runs'}`;

const buildPeriod = (
  label: string,
  detail: string,
  counters: TaskBoardUsageCounters,
  emptyMessage: string
): TaskBoardProjectUsagePeriodData => ({
  label,
  detail,
  tokensLabel: `${formatTokenCount(counters.tokensUsed)} tokens`,
  finishedRunsLabel: finishedRunsLabel(counters.finishedRunsCount),
  breakdownLabel: `${label} token breakdown`,
  counters,
  emptyMessage: hasUsage(counters) ? null : emptyMessage
});

export const buildProjectUsageSummary = (
  usage: TaskBoardProjectUsage | undefined
): TaskBoardProjectUsageSummaryData | undefined => {
  if (!usage) return undefined;

  return {
    emptyMessage: hasUsage(usage.total)
      ? null
      : 'No accepted run usage recorded for this project yet.',
    total: buildPeriod(
      'Total',
      'All accepted runs',
      usage.total,
      'No accepted run usage recorded for this project yet.'
    ),
    thisWeek: buildPeriod(
      'This week',
      'Monday 00:00 UTC to now',
      usage.thisWeek,
      'No usage this week.'
    )
  };
};
