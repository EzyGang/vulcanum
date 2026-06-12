import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { formatTokenCount } from '../../../../utils/format';
import { Tooltip } from '../../../shared/ui/Tooltip.view';

interface UsageStatProps {
  icon: string;
  label: string;
  value: number | null | undefined;
}

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

const UsageTooltipContent = ({ run }: { run: WorkRunListItem }): JSX.Element => (
  <div class='flex flex-col gap-2 font-mono'>
    <div class='grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-1'>
      <span class='text-text-muted'>↓</span>
      <span>Input tokens</span>
      <span class='text-text-primary'>{formatTokenCount(run.inputTokens)}</span>
      <span class='text-text-muted'>↑</span>
      <span>Output tokens</span>
      <span class='text-text-primary'>{formatTokenCount(run.outputTokens)}</span>
      <span class='text-text-muted'>↙</span>
      <span>Cache read tokens</span>
      <span class='text-text-primary'>{formatTokenCount(run.cacheReadTokens)}</span>
      <span class='text-text-muted'>↗</span>
      <span>Cache write tokens</span>
      <span class='text-text-primary'>{formatTokenCount(run.cacheWriteTokens)}</span>
    </div>
    {run.modelUsed && (
      <div class='border-t border-border-base pt-2 text-text-muted'>
        Model: <span class='text-text-secondary'>{run.modelUsed}</span>
      </div>
    )}
  </div>
);

export const hasRunUsageStats = (run: WorkRunListItem): boolean =>
  (run.inputTokens !== null && run.inputTokens !== undefined) ||
  (run.outputTokens !== null && run.outputTokens !== undefined) ||
  (run.cacheReadTokens !== null && run.cacheReadTokens !== undefined) ||
  (run.cacheWriteTokens !== null && run.cacheWriteTokens !== undefined);

export const RunUsageStats = ({ run }: { run: WorkRunListItem }): JSX.Element => (
  <Tooltip>
    <Tooltip.Trigger class='block bg-transparent p-0 text-left font-mono text-xs'>
      <div class='flex max-w-96 flex-col gap-1'>
        <div class='flex flex-wrap items-center gap-1'>
          <UsageStat icon='↓' label='Input tokens' value={run.inputTokens} />
          <UsageStat icon='↑' label='Output tokens' value={run.outputTokens} />
          <UsageStat icon='↙' label='Cache read tokens' value={run.cacheReadTokens} />
          <UsageStat icon='↗' label='Cache write tokens' value={run.cacheWriteTokens} />
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
