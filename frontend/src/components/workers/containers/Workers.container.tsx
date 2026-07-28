import type { JSX } from 'preact';
import { useWorkers } from '../hooks/useWorkers.hook';
import { WorkersView } from '../ui/Workers.view';

export const WorkersContainer = (): JSX.Element => {
  const {
    formattedWorkers,
    maskedCode,
    setupCommandPreview,
    countdown,
    generateLoading,
    copiedTarget,
    copyError,
    deletingId,
    deleteError,
    updateStatusError,
    renameError,
    loading,
    error,
    handleGenerateCode,
    handleConfirmDelete,
    handleCancelDelete,
    handleDeleteWorker,
    handleUpdateStatus,
    handleRenameWorker,
    copyGeneratedCode,
    copySetupCommand
  } = useWorkers();

  return (
    <WorkersView
      data={{ workers: formattedWorkers, maskedCode, setupCommandPreview, countdown }}
      status={{
        loading,
        error,
        generateLoading,
        deletingId,
        deleteError,
        updateStatusError,
        copiedTarget,
        renameError,
        copyError
      }}
      actions={{
        onGenerateCode: handleGenerateCode,
        onConfirmDelete: handleConfirmDelete,
        onCancelDelete: handleCancelDelete,
        onDeleteWorker: handleDeleteWorker,
        onUpdateStatus: handleUpdateStatus,
        onCopyCode: copyGeneratedCode,
        onCopySetupCommand: copySetupCommand,
        onRenameWorker: handleRenameWorker
      }}
    />
  );
};
