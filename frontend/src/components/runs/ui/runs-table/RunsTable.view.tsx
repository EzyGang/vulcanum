import type { Signal } from '@preact/signals';
import { IconChevronDown, IconChevronRight } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import {
  CANCELLABLE_STATUSES,
  WORK_RUN_PR_LINK_LABELS,
  WORK_RUN_TYPE_LABELS
} from '../../../../types/runs';
import { formatDuration, formatRelativeTime } from '../../../../utils/format';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import { RunEventTimelineContainer } from '../../containers/run-events/RunEventTimeline.container';
import { RunActions } from './RunActions.view';
import { RunTaskCell } from './RunTaskCell.view';
import { hasRunUsageStats, RunUsageStats } from './RunUsageStats';

const COL_SPAN = 9;

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
  onStopRowToggle
}: RunsTableProps): JSX.Element => (
  <Table>
    <Table.Head>
      <Table.HeadCell class='w-5 px-1'>{''}</Table.HeadCell>
      <Table.HeadCell class='w-5 px-1! py-3!'>
        <Checkbox
          checked={allSelected}
          indeterminate={someSelected}
          onCheckedChange={onToggleSelectAll}
        />
      </Table.HeadCell>
      <Table.HeadCell>Task</Table.HeadCell>
      <Table.HeadCell>Run</Table.HeadCell>
      <Table.HeadCell class='hidden lg:table-cell'>Execution</Table.HeadCell>
      <Table.HeadCell class='hidden lg:table-cell'>Tokens</Table.HeadCell>
      <Table.HeadCell class='hidden lg:table-cell'>PR</Table.HeadCell>
      <Table.HeadCell class='hidden lg:table-cell'>Created</Table.HeadCell>
      <Table.HeadCell>Actions</Table.HeadCell>
    </Table.Head>
    <Table.Body>
      {runs.map((run) => {
        const expanded = expandedIds.value.has(run.id);
        const cancellable = CANCELLABLE_STATUSES.includes(run.status);
        return (
          <>
            <Table.Row key={run.id} class='cursor-pointer' onClick={() => onToggleExpanded(run.id)}>
              <Table.Cell class='w-1' paddingClass='px-1 py-3'>
                <span class='text-text-muted'>
                  {expanded ? (
                    <IconChevronDown size={16} stroke={1.75} aria-hidden='true' />
                  ) : (
                    <IconChevronRight size={16} stroke={1.75} aria-hidden='true' />
                  )}
                </span>
              </Table.Cell>
              <Table.Cell onClick={onStopRowToggle} paddingClass='px-1 py-3'>
                <Checkbox
                  checked={selectedIds.value.has(run.id)}
                  onCheckedChange={() => onToggleSelect(run.id)}
                />
              </Table.Cell>
              <Table.Cell paddingClass='pl-5 py-3'>
                <RunTaskCell run={run} />
              </Table.Cell>
              <Table.Cell>
                <div class='flex flex-col items-start gap-2'>
                  <StatusBadge status={run.status} />
                  <span class='border border-border-base bg-bg-panel px-2 py-1 text-xs font-mono text-text-secondary'>
                    {WORK_RUN_TYPE_LABELS[run.workType]}
                  </span>
                </div>
              </Table.Cell>
              <Table.Cell class='hidden lg:table-cell'>
                <div class='flex flex-col gap-1'>
                  <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
                  <span class='text-text-muted text-xs font-mono tabular-nums'>
                    {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
                  </span>
                </div>
              </Table.Cell>
              <Table.Cell class='hidden lg:table-cell' onClick={onStopRowToggle}>
                {hasRunUsageStats(run) ? (
                  <RunUsageStats run={run} />
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden lg:table-cell'>
                {run.reviewTargetPrUrl || run.resultPrUrl ? (
                  <a
                    href={run.reviewTargetPrUrl ?? run.resultPrUrl ?? ''}
                    target='_blank'
                    rel='noopener noreferrer'
                    onClick={onStopRowToggle}
                    class='text-accent text-sm hover:underline'
                  >
                    {WORK_RUN_PR_LINK_LABELS[run.workType]}
                  </a>
                ) : (
                  <span class='text-text-muted text-sm'>—</span>
                )}
              </Table.Cell>
              <Table.Cell class='hidden lg:table-cell'>
                <span class='text-text-secondary text-sm'>{formatRelativeTime(run.createdAt)}</span>
              </Table.Cell>
              <Table.Cell onClick={onStopRowToggle}>
                <RunActions
                  runId={run.id}
                  cancellable={cancellable}
                  deleting={deletingId.value === run.id}
                  onFailRun={onFailRun}
                  onCancelRun={onCancelRun}
                  onConfirmDelete={onConfirmDelete}
                  onDelete={onDelete}
                  onCancelDelete={onCancelDelete}
                />
              </Table.Cell>
            </Table.Row>
            {expanded && (
              <Table.Row key={`${run.id}-events`}>
                <Table.Cell colSpan={COL_SPAN} paddingClass='p-0'>
                  <div class='p-2'>
                    <RunEventTimelineContainer runId={run.id} status={run.status} />
                  </div>
                </Table.Cell>
              </Table.Row>
            )}
          </>
        );
      })}
    </Table.Body>
  </Table>
);
