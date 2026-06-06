import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem } from '../../../../types/runs';
import { formatDuration, formatRelativeTime } from '../../../../utils/format';
import { Button } from '../../../shared/ui/Button.view';
import { Checkbox } from '../../../shared/ui/Checkbox.view';
import { ConfirmDelete } from '../../../shared/ui/ConfirmDelete.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';
import { Table } from '../../../shared/ui/Table.view';
import { RunEventTimelineContainer } from '../../containers/run-events/RunEventTimeline.container';

interface RunsTableRowProps {
  run: WorkRunListItem;
  selectedIds: Signal<Set<string>>;
  deletingId: Signal<string | null>;
  expandedIds: Signal<Set<string>>;
  onToggleSelect: (id: string) => void;
  onToggleExpanded: (id: string) => void;
  onCancelRun: (id: string) => void;
  onFailRun: (id: string) => void;
  onConfirmDelete: (id: string) => void;
  onDelete: (id: string) => void;
  onCancelDelete: () => void;
}

const isCancellable = (status: WorkRunListItem['status']): boolean =>
  status === 'running' || status === 'dispatched';

export const RunsTableRow = ({
  run,
  selectedIds,
  deletingId,
  expandedIds,
  onToggleSelect,
  onToggleExpanded,
  onCancelRun,
  onFailRun,
  onConfirmDelete,
  onDelete,
  onCancelDelete
}: RunsTableRowProps): JSX.Element => {
  const expanded = expandedIds.value.has(run.id);
  const cancellable = isCancellable(run.status);

  return (
    <>
      <Table.Row key={run.id}>
        <Table.Cell>
          <button
            type='button'
            aria-label={expanded ? 'Collapse' : 'Expand'}
            onClick={() => onToggleExpanded(run.id)}
            class='text-text-muted hover:text-text-primary text-xs px-1'
          >
            {expanded ? '▾' : '▸'}
          </button>
        </Table.Cell>
        <Table.Cell>
          <Checkbox
            checked={selectedIds.value.has(run.id)}
            onCheckedChange={() => onToggleSelect(run.id)}
          />
        </Table.Cell>
        <Table.Cell>
          <span class='text-text-primary text-sm font-mono'>{run.externalTaskRef}</span>
        </Table.Cell>
        <Table.Cell>
          <StatusBadge status={run.status} />
        </Table.Cell>
        <Table.Cell>
          <span class='text-text-secondary text-sm'>{run.workerName ?? '—'}</span>
        </Table.Cell>
        <Table.Cell>
          <span class='text-text-secondary text-sm font-mono'>
            {run.durationMs !== null ? formatDuration(run.durationMs) : '—'}
          </span>
        </Table.Cell>
        <Table.Cell>
          {run.resultPrUrl ? (
            <a
              href={run.resultPrUrl}
              target='_blank'
              rel='noopener noreferrer'
              class='text-accent text-sm hover:underline'
            >
              PR
            </a>
          ) : (
            <span class='text-text-muted text-sm'>—</span>
          )}
        </Table.Cell>
        <Table.Cell>
          <span class='text-text-secondary text-sm'>{formatRelativeTime(run.createdAt)}</span>
        </Table.Cell>
        <Table.Cell>
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
          <td colSpan={9} class='p-0'>
            <div class='p-2'>
              <RunEventTimelineContainer runId={run.id} status={run.status} />
            </div>
          </td>
        </Table.Row>
      )}
    </>
  );
};
