import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem, WorkRunStatus } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { formatDuration, formatRelativeTime } from '../../../utils/format';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { StatusBadge } from '../../shared/ui/StatusBadge.view';
import { Table } from '../../shared/ui/Table.view';
import { RunFilterBar } from './RunFilterBar.view';
import { RunPagination } from './RunPagination.view';

interface RunsViewProps {
  data: {
    runs: WorkRunListItem[];
  };
  status: {
    loading: boolean;
    error: ApiError | null;
    deleteError: Signal<string | null>;
    deletingId: Signal<string | null>;
    statusFilter: Signal<WorkRunStatus | undefined>;
    page: Signal<number>;
    hasNextPage: boolean;
    hasPrevPage: boolean;
  };
  actions: {
    setStatusFilter: (value: string) => void;
    nextPage: () => void;
    prevPage: () => void;
    handleDeleteRun: (id: string) => void;
    handleConfirmDelete: (id: string) => void;
    handleCancelDelete: () => void;
  };
}

export const RunsView = ({
  data: { runs },
  status: { loading, error, deleteError, deletingId, statusFilter, page, hasNextPage, hasPrevPage },
  actions: {
    setStatusFilter,
    nextPage,
    prevPage,
    handleDeleteRun,
    handleConfirmDelete,
    handleCancelDelete
  }
}: RunsViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <div class='flex items-center justify-between'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Work Runs</h2>
      <RunFilterBar statusFilter={statusFilter} onStatusFilter={setStatusFilter} />
    </div>

    {error && <ErrorBanner message={error.message} />}
    {deleteError.value && <ErrorBanner message={deleteError.value} />}
    {loading && <div class='text-text-muted text-sm'>Loading runs...</div>}
    {!loading && !error && runs.length === 0 && <EmptyState title='No work runs found.' />}

    {runs.length > 0 && (
      <div class='overflow-x-auto'>
        <table class='w-full border-collapse'>
          <Table.Head>
            <Table.HeadCell>Task</Table.HeadCell>
            <Table.HeadCell>Status</Table.HeadCell>
            <Table.HeadCell>Worker</Table.HeadCell>
            <Table.HeadCell>Duration</Table.HeadCell>
            <Table.HeadCell>PR</Table.HeadCell>
            <Table.HeadCell>Created</Table.HeadCell>
            <Table.HeadCell>Actions</Table.HeadCell>
          </Table.Head>
          <Table.Body>
            {runs.map((run) => (
              <Table.Row key={run.id}>
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
                  <span class='text-text-secondary text-sm'>
                    {formatRelativeTime(run.createdAt)}
                  </span>
                </Table.Cell>
                <Table.Cell>
                  <ConfirmDelete
                    itemId={run.id}
                    deletingId={deletingId}
                    onConfirm={handleConfirmDelete}
                    onDelete={handleDeleteRun}
                    onCancel={handleCancelDelete}
                  />
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </table>

        <RunPagination
          page={page}
          hasPrevPage={hasPrevPage}
          hasNextPage={hasNextPage}
          loading={loading}
          onPrev={prevPage}
          onNext={nextPage}
        />
      </div>
    )}
  </div>
);
