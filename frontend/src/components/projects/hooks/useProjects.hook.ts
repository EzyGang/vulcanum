import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import { listModelProviders } from '../../../services/model-providers/model-providers.service';
import { deleteProject, listProjects } from '../../../services/projects/projects.service';
import { listProviders } from '../../../services/providers/providers.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import {
  getProjectSetupHelpText,
  getProjectSetupMissingMessages,
  isProjectSetupComplete
} from '../../../utils/projectSetup';

export const useProjects = () => {
  const {
    data: projects,
    isLoading: loading,
    error
  } = useApiQuery(['projects'], () => listProjects());

  const { data: providers = [], isLoading: providersLoading } = useApiQuery(['providers'], () =>
    listProviders()
  );
  const { data: modelProviders = [], isLoading: modelProvidersLoading } = useApiQuery(
    ['model-providers'],
    () => listModelProviders()
  );

  const setupState = {
    hasTaskTrackerProvider: providers.length > 0,
    hasModelProvider: modelProviders.length > 0
  };
  const setupLoading = providersLoading || modelProvidersLoading;
  const setupMessages = getProjectSetupMissingMessages(setupState);

  const deleteMutation = useApiMutation((id: string) => deleteProject(id), {
    onSuccess: () => invalidate('projects')
  });

  const {
    deletingId: deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  } = useDeleteConfirm('project config', deleteMutation);

  return {
    projects: projects ?? [],
    canCreateProject: !setupLoading && isProjectSetupComplete(setupState),
    projectSetupWarning: setupLoading ? '' : getProjectSetupHelpText(setupMessages),
    loading,
    error,
    deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  };
};
