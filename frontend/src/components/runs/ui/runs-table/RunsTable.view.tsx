import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { CANCELLABLE_STATUSES } from '../../../../types/runs';
import { formatDuration, formatRelativeTime, formatTokenCount } from '../../../../utils/format';
import { Button } from '../../../shared/ui/Button.view';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { ConfirmDelete } from '../../../shared/ui/ConfirmDelete.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import { Tooltip } from '../../../shared/ui/Tooltip.view';
import { RunEventTimelineContainer } from '../../containers/run-events/RunEventTimeline.container';

const COL_SPAN = 10;

const hasUsageStats = (run: WorkRunListItem): boolean =>
  (run.inputTokens !== null && run.inputTokens !== undefined) ||
  (run.outputTokens !== null && run.outputTokens !== undefined) ||
  (run.cacheReadTokens !== null && run.cacheReadTokens !== undefined) ||
  (run.cacheWriteTokens !== null && run.cacheWriteTokens !== undefined);

const stopRowToggle = (event: JSX.TargetedMouseEvent<HTMLElement>): void => {
  event.stopPropagation();
};

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

const UsageStats = ({ run }: { run: WorkRunListItem }): JSX.Element => (
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

interface RunsTableProps {
  runs: WorkRunListItem[];
  selectedIds: Signal<Set<string>>;
  allSelected: boolean;
  someSelected: boolean;
  expandedIds: Signal<Set<string>>;
  deletingId: Signal<string | null>;
  onToggleSelect: (id: string) => void;
  onToggleSelectAll: () => void;
  onToggleExpanded: (id: string) => void;
  onFailRun: (id: string) => void;
  onCancelRun: (id: string) => void;
  onConfirmDelete: (id: string) => void;
  onDelete: (id: string) => void;
  onCancelDelete: () => void;
}

export const RunsTable = ({
  runs,
  selectedIds,
  allSelected,
  someSelected,
  expandedIds,
  deletingId,
  onToggleSelect,
  onToggleSelectAll,
  onToggleExpanded,
  onFailRun,
  onCancelRun,
  onConfirmDelete,
  onDelete,
  onCancelDelete
}: RunsTableProps): JSX.Element => (
  <table class='w-full border-collapse'>
    <Table.Head>
      <Table.HeadCell class='w-5 px-1'>{''}</Table.HeadCell>
      <Table.HeadCell class='w-10'>
        <Checkbox
          checked={allSelected}
          indeterminate={someSelected}
          onCheckedChange={onToggleSelectAll}
        />
      </Table.HeadCell>
      <Table.HeadCell>Task</Table.HeadCell>
      <Table.HeadCell>Status</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Worker</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Duration</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Tokens</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>PR</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Created</Table.HeadCell>
      <Table.HeadCell class='hidden md:table-cell'>Actions</Table.HeadCell>
    </Table.Head>
    <Table.Body>
      {runs.map((run) => {
        const expanded = expandedIds.value.has(run.id);
        const cancellable = CANCELLABLE_STATUSES.includes(run.status);
        return (
          <>
            <Table.Row key={run.id} class='cursor-pointer' onClick={() => onToggleExpanded(run.id)}>
              <Table.Cell class='w-5 px-1'>
                <button
                  type='button'
                  aria-label={expanded ? 'Collapse' : 'Expand'}
                  onClick={(event) => {
                    event.stopPropagation();
                    onToggleExpanded(run.id);
                  }}
                  class='px-1 text-xs text-text-muted hover:text-text-primary'
                >
                  {expanded ? '▾' : '▸'}
                </button>
              </Table.Cell>
              <Table.Cell onClick={stopRowToggle}>
                <Checkbox
                  checked={selectedIds.value.has(run.id)}
                  onCheckedChange={() => onToggleSelect(run.id)}
                />
              </Table.Cell>
              <Table.Cell>
                {run.taskSlug ? (
                  <span class='text-text-primary text-sm font-mono'>{run.taskSlug}</span>
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
                {run.taskTitle && (
                  <span class='text-text-secondary text-sm ml-2 truncate max-w-48 inline-block align-middle'>
                    {run.taskTitle}
                  </span>
                )}
              </Table.Cell>
              <Table.Cell>
                <StatusBadge status={run.status} />
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm font-mono'>
                  {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
                </span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell' onClick={stopRowToggle}>
                {hasUsageStats(run) ? (
                  <UsageStats run={run} />
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                {run.resultPrUrl ? (
                  <a
                    href={run.resultPrUrl}
                    target='_blank'
                    rel='noopener noreferrer'
                    onClick={stopRowToggle}
                    class='text-accent text-sm hover:underline'
                  >
                    PR
                  </a>
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm'>{formatRelativeTime(run.createdAt)}</span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell' onClick={stopRowToggle}>
                <div class='flex items-center gap-2'>
                  {cancellable && (
                    <Button variant='ghost' onClick={() => onCancelRun(run.id)}>
                      Cancel
                    </Button>
                  )}
                  {cancellable && (
                    <Button variant='ghost-danger' onClick={() => onFailRun(run.id)}>
                      Fail
                    </Button>
                  )}
                  <ConfirmDelete
                    itemId={run.id}
                    deletingId={deletingId}
                    onConfirm={onConfirmDelete}
                    onDelete={onDelete}
                    onCancel={onCancelDelete}
                  />
                </div>
              </Table.Cell>
            </Table.Row>
            {expanded && (
              <Table.Row key={`${run.id}-events`}>
                <td colSpan={COL_SPAN} class='p-0'>
                  <div class='p-2'>
                    <RunEventTimelineContainer runId={run.id} status={run.status} />
                  </div>
                </td>
              </Table.Row>
            )}
          </>
        );
      })}
    </Table.Body>
  </table>
);
