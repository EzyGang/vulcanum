import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { CANCELLABLE_STATUSES } from '../../../../types/runs';
import { formatDuration, formatRelativeTime } from '../../../../utils/format';
import { Button } from '../../../shared/ui/Button.view';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { ConfirmDelete } from '../../../shared/ui/ConfirmDelete.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import { RunEventTimelineContainer } from '../../containers/run-events/RunEventTimeline.container';
import { hasRunUsageStats, RunUsageStats } from './RunUsageStats';

const COL_SPAN = 11;

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
  onStopRowToggle: (event: JSX.TargetedMouseEvent<HTMLElement>) => void;
  onToggleExpandedControl: (id: string, event: JSX.TargetedMouseEvent<HTMLElement>) => void;
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
  onCancelDelete,
  onStopRowToggle,
  onToggleExpandedControl
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
      <Table.HeadCell class='hidden md:table-cell'>Type</Table.HeadCell>
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
                  onClick={(event) => onToggleExpandedControl(run.id, event)}
                  class='px-1 text-xs text-text-muted hover:text-text-primary'
                >
                  {expanded ? '▾' : '▸'}
                </button>
              </Table.Cell>
              <Table.Cell onClick={onStopRowToggle}>
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
                <span class='border border-border-base bg-bg-panel px-2 py-1 text-xs font-mono text-text-secondary'>
                  {run.workType === 'pull_request_review' ? 'Review' : 'Implement'}
                </span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm font-mono'>
                  {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
                </span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell' onClick={onStopRowToggle}>
                {hasRunUsageStats(run) ? (
                  <RunUsageStats run={run} />
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                {run.reviewTargetPrUrl || run.resultPrUrl ? (
                  <a
                    href={run.reviewTargetPrUrl ?? run.resultPrUrl ?? ''}
                    target='_blank'
                    rel='noopener noreferrer'
                    onClick={onStopRowToggle}
                    class='text-accent text-sm hover:underline'
                  >
                    {run.workType === 'pull_request_review' ? 'Review PR' : 'PR'}
                  </a>
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell'>
                <span class='text-text-secondary text-sm'>{formatRelativeTime(run.createdAt)}</span>
              </Table.Cell>
              <Table.Cell class='hidden md:table-cell' onClick={onStopRowToggle}>
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
