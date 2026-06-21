import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { hasRunUsageStats, RunUsageStats } from './RunUsageStats';

interface RunTaskCellProps {
  run: WorkRunListItem;
}

export const RunTaskCell = ({ run }: RunTaskCellProps): JSX.Element => (
  <div class='flex flex-col gap-2'>
    <div class='flex flex-col'>
      {run.taskSlug ? (
        <span class='text-text-primary text-sm font-mono'>{run.taskSlug}</span>
      ) : (
        <span class='text-text-muted text-sm'>—</span>
      )}
      {run.taskTitle && (
        <span class='text-text-secondary text-sm truncate max-w-48 inline-block align-middle'>
          {run.taskTitle}
        </span>
      )}
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
