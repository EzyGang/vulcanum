import { Fragment, type JSX } from 'preact';
import { formatTokenCount } from '../../../../utils/format';
import { Tooltip } from '../../../shared/ui/Tooltip.view';

interface UsageStatProps {
  icon: string;
  label: string;
  value: number | null | undefined;
}

type UsageTokenField = 'inputTokens' | 'outputTokens' | 'cacheReadTokens' | 'cacheWriteTokens';

interface UsageStatConfig {
  field: UsageTokenField;
  icon: string;
  label: string;
}

export interface RunUsageStatsData {
  inputTokens?: number | null;
  outputTokens?: number | null;
  cacheReadTokens?: number | null;
  cacheWriteTokens?: number | null;
  modelUsed?: string | null;
}

export const RUN_USAGE_STATS = [
  { field: 'inputTokens', icon: '↑', label: 'Input tokens' },
  { field: 'outputTokens', icon: '↓', label: 'Output tokens' },
  { field: 'cacheReadTokens', icon: '↙', label: 'Cache read tokens' },
  { field: 'cacheWriteTokens', icon: '↗', label: 'Cache write tokens' }
] as const satisfies readonly UsageStatConfig[];

const UsageStat = ({ icon, label, value }: UsageStatProps): JSX.Element => (
  <span class='inline-flex items-center gap-1 border border-border-base bg-bg-card px-1.5 py-0.5 text-text-secondary'>
    <span class='sr-only'>
      {label}: {formatTokenCount(value)}
    </span>
    <span class='text-text-muted' aria-hidden='true'>
      {icon}
    </span>
    <span>{formatTokenCount(value)}</span>
  </span>
);

const UsageTooltipContent = ({ run }: { run: RunUsageStatsData }): JSX.Element => (
  <div class='flex flex-col gap-2 font-mono'>
    <div class='grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-1'>
      {RUN_USAGE_STATS.map((stat) => (
        <Fragment key={stat.field}>
          <span class='text-text-muted'>{stat.icon}</span>
          <span>{stat.label}</span>
          <span class='text-text-primary'>{formatTokenCount(run[stat.field])}</span>
        </Fragment>
      ))}
    </div>
    {run.modelUsed && (
      <div class='border-t border-border-base pt-2 text-text-muted'>
        Model: <span class='text-text-secondary'>{run.modelUsed}</span>
      </div>
    )}
  </div>
);

export const hasRunUsageStats = (run: RunUsageStatsData): boolean =>
  RUN_USAGE_STATS.some((stat) => run[stat.field] !== null && run[stat.field] !== undefined);

export const RunUsageStats = ({ run }: { run: RunUsageStatsData }): JSX.Element => (
  <Tooltip>
    <Tooltip.Trigger class='block bg-transparent p-0 text-left font-mono text-xs'>
      <div class='flex max-w-96 flex-col gap-1'>
        <div class='flex flex-wrap items-center gap-1'>
          {RUN_USAGE_STATS.map((stat) => (
            <UsageStat
              key={stat.field}
              icon={stat.icon}
              label={stat.label}
              value={run[stat.field]}
            />
          ))}
        </div>
        {run.modelUsed && (
          <span class='block max-w-72 truncate text-text-muted xl:max-w-96' title={run.modelUsed}>
            {run.modelUsed}
          </span>
        )}
      </div>
    </Tooltip.Trigger>
    <Tooltip.Popup>
      <UsageTooltipContent run={run} />
    </Tooltip.Popup>
  </Tooltip>
);
