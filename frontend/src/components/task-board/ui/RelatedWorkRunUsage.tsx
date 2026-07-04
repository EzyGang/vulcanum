import type { JSX } from 'preact';
import type { TaskBoardRelatedWorkRun } from '../../../types/task-board';
import { formatTokenCount } from '../../../utils/format';
import { hasRunUsageStats, RunUsageStats } from '../../runs/ui/runs-table/RunUsageStats';

export const RelatedWorkRunUsage = ({ run }: { run: TaskBoardRelatedWorkRun }): JSX.Element => {
  if (hasRunUsageStats(run)) {
    return <RunUsageStats run={run} />;
  }

  if (run.tokensUsed !== null) {
    return (
      <span class='font-mono text-xs tabular-nums text-text-secondary'>
        {formatTokenCount(run.tokensUsed)} tokens
      </span>
    );
  }

  return <span class='text-xs text-text-muted'>No usage recorded</span>;
};
