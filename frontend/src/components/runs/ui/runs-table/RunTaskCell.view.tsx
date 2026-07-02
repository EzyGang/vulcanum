import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { hasRunUsageStats, RunUsageStats } from './RunUsageStats';

interface RunTaskCellProps {
  run: WorkRunListItem;
}

export const RunTaskCell = ({ run }: RunTaskCellProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <div class='flex flex-col'>
      <span class='text-text-primary text-sm font-mono'>{run.externalTaskRef}</span>
    </div>
    <div class='lg:hidden'>
      {hasRunUsageStats(run) ? (
        <RunUsageStats run={run} />
      ) : (
        <span class='text-text-muted text-sm'>—</span>
      )}
    </div>
  </div>
);
