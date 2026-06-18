import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { WorkRunListItem, WorkRunStatus } from '../../../types/runs';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { Dialog } from '../../shared/ui/Dialog.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { RunFilterBar } from './run-controls/RunFilterBar.view';
import { RunPagination } from './run-controls/RunPagination.view';
import { RunsTable } from './runs-table/RunsTable.view';

interface RunsViewProps {
  data: {
    runs: WorkRunListItem[];
    selectedIds: Signal<Set<string>>;
    allSelected: boolean;
    someSelected: boolean;
    selectionCount: number;
    showBulkDeleteDialog: Signal<boolean>;
    expandedIds: Signal<Set<string>>;
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
    handleToggleSelect: (id: string) => void;
    handleToggleSelectAll: () => void;
    handleOpenBulkDelete: () => void;
    handleConfirmBulkDelete: () => void;
    handleCancelBulkDelete: () => void;
    handleFailRun: (id: string) => void;
    handleToggleExpanded: (id: string) => void;
    handleStopRowToggle: (event: JSX.TargetedMouseEvent<HTMLElement>) => void;
    handleCancelRun: (id: string) => void;
  };
}

export const RunsView = ({
  data: {
    runs,
    selectedIds,
    allSelected,
    someSelected,
    selectionCount,
    showBulkDeleteDialog,
    expandedIds
  },
  status: { loading, error, deleteError, deletingId, statusFilter, page, hasNextPage, hasPrevPage },
  actions: {
    setStatusFilter,
    nextPage,
    prevPage,
    handleDeleteRun,
    handleConfirmDelete,
    handleCancelDelete,
    handleToggleSelect,
    handleToggleSelectAll,
    handleOpenBulkDelete,
    handleConfirmBulkDelete,
    handleCancelBulkDelete,
    handleFailRun,
    handleToggleExpanded,
    handleStopRowToggle,
    handleCancelRun
  }
}: RunsViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <div class='flex items-center justify-between'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Work Runs</h2>
      <RunFilterBar statusFilter={statusFilter.value} onStatusFilter={setStatusFilter} />
    </div>

    {error && <ErrorBanner message={error.message} />}
    {deleteError.value && <ErrorBanner message={deleteError.value} />}
    {loading && <div class='text-text-muted text-sm'>Loading runs...</div>}
    {!loading && !error && runs.length === 0 && <EmptyState title='No work runs found.' />}

    {selectionCount > 0 && (
      <div class='flex items-center gap-3 px-1'>
        <span class='text-text-secondary text-sm'>{selectionCount} selected</span>
        <Button variant='ghost-danger' onClick={handleOpenBulkDelete}>
          Delete Selected
        </Button>
      </div>
    )}

    {runs.length > 0 && (
      <div class='overflow-x-auto'>
        <RunsTable
          runs={runs}
          selectedIds={selectedIds}
          allSelected={allSelected}
          someSelected={someSelected}
          expandedIds={expandedIds}
          deletingId={deletingId}
          onToggleSelect={handleToggleSelect}
          onToggleSelectAll={handleToggleSelectAll}
          onToggleExpanded={handleToggleExpanded}
          onFailRun={handleFailRun}
          onCancelRun={handleCancelRun}
          onConfirmDelete={handleConfirmDelete}
          onDelete={handleDeleteRun}
          onCancelDelete={handleCancelDelete}
          onStopRowToggle={handleStopRowToggle}
        />

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

    {showBulkDeleteDialog.value && (
      <Dialog
        open
        onOpenChange={(open) => {
          if (!open) handleCancelBulkDelete();
        }}
      >
        <Dialog.Backdrop />
        <Dialog.Portal>
          <Dialog.Popup>
            <div class='flex flex-col gap-4'>
              <Dialog.Title>
                Delete {selectionCount} run{selectionCount !== 1 ? 's' : ''}?
              </Dialog.Title>
              <Dialog.Description>
                This action cannot be undone. Running runs will be skipped.
              </Dialog.Description>
              <div class='flex justify-end gap-3'>
                <Dialog.Close>
                  <Button variant='ghost'>Cancel</Button>
                </Dialog.Close>
                <Button variant='danger' onClick={handleConfirmBulkDelete}>
                  Delete
                </Button>
              </div>
            </div>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog>
    )}
  </div>
);
