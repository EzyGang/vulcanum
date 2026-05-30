import type { JSX } from 'preact';
import { useWorkers } from '../hooks/useWorkers.hook';
import { WorkersView } from '../ui/Workers.view';

export const WorkersContainer = (): JSX.Element => {
  const {
    formattedWorkers,
    code,
    countdown,
    generateLoading,
    deletingId,
    deleteError,
    updateStatusError,
    loading,
    error,
    handleGenerateCode,
    handleConfirmDelete,
    handleCancelDelete,
    handleDeleteWorker,
    handleUpdateStatus
  } = useWorkers();

  return (
    <WorkersView
      data={{ workers: formattedWorkers, code, countdown }}
      status={{ loading, error, generateLoading, deletingId, deleteError, updateStatusError }}
      actions={{
        onGenerateCode: handleGenerateCode,
        onConfirmDelete: handleConfirmDelete,
        onCancelDelete: handleCancelDelete,
        onDeleteWorker: handleDeleteWorker,
        onUpdateStatus: handleUpdateStatus
      }}
    />
  );
};
