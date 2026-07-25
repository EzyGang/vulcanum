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
      <div class='grid min-w-0 gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(10rem,0.75fr)] lg:items-start'>
        <div class='flex min-w-0 flex-col gap-2'>
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
        </div>
        <div
          role='group'
          aria-label={data.runCountsLabel}
          class='flex min-w-0 flex-col gap-2 border-t border-border-base pt-3 lg:border-t-0 lg:border-l lg:pt-0 lg:pl-4'
        >
          <span class='text-xs font-medium uppercase tracking-wider text-text-muted'>Runs</span>
          <dl class='grid grid-cols-2 gap-x-4 gap-y-2 lg:grid-cols-1'>
            {data.runCountStats.map((stat) => (
              <div
                key={stat.label}
                class='flex min-w-0 items-baseline justify-between gap-2 border-b border-border-base pb-1.5'
              >
                <dt class='text-xs text-text-muted'>{stat.label}</dt>
                <dd class='shrink-0 font-mono text-sm font-medium tabular-nums text-text-primary'>
                  {stat.valueLabel}
                </dd>
              </div>
            ))}
          </dl>
        </div>
      </div>
    )}
  </div>
);
