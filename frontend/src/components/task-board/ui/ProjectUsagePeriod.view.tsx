import type { JSX } from 'preact';
import { RunUsageStats } from '../../runs/ui/runs-table/RunUsageStats';
import type { TaskBoardProjectUsagePeriodViewProps } from '../types';

export const ProjectUsagePeriodView = ({
  data
}: TaskBoardProjectUsagePeriodViewProps): JSX.Element => (
  <div class='flex min-w-0 flex-col gap-3 p-4'>
    <div class='flex flex-col gap-1'>
      <span class='text-xs font-medium uppercase tracking-wider text-text-muted'>{data.label}</span>
      <span class='text-xs text-text-muted'>{data.detail}</span>
    </div>
    {data.emptyMessage ? (
      <p class='text-sm text-text-muted'>{data.emptyMessage}</p>
    ) : (
      <>
        <div class='flex flex-wrap items-baseline gap-x-3 gap-y-1'>
          <span class='font-mono text-xl font-semibold tabular-nums text-text-primary'>
            {data.tokensLabel}
          </span>
          <span class='font-mono text-xs tabular-nums text-text-muted'>
            {data.finishedRunsLabel}
          </span>
        </div>
        <div role='group' aria-label={data.breakdownLabel}>
          <RunUsageStats run={data.counters} />
        </div>
        <div class='flex flex-col gap-2'>
          <span class='text-xs font-medium uppercase tracking-wider text-text-muted'>Runs</span>
          <div role='group' aria-label={data.runCountsLabel} class='grid grid-cols-2 gap-2'>
            {data.runCountStats.map((stat) => (
              <div
                key={stat.label}
                class='flex items-center justify-between gap-3 border border-border-base bg-bg-card px-2.5 py-2'
              >
                <span class='text-xs text-text-muted'>{stat.label}</span>
                <span class='font-mono text-sm font-medium tabular-nums text-text-primary'>
                  {stat.valueLabel}
                </span>
              </div>
            ))}
          </div>
        </div>
      </>
    )}
  </div>
);
