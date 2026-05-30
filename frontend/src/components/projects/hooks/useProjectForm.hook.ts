import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import {
  createProject,
  getProject,
  updateProject
} from '../../../services/projects/projects.service';
import { listProviders, lookupProject } from '../../../services/providers/providers.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { useProjectFormLookup } from './useProjectFormLookup.hook';
import { useProjectFormProvider } from './useProjectFormProvider.hook';

export const useProjectForm = (projectId: string | null) => {
  const { data: existingProject, isLoading: projectLoading } = useApiQuery(
    ['project', projectId ?? ''],
    () => getProject(projectId ?? '')
  );

  const { data: providers = [] } = useApiQuery(['providers'], () => listProviders());

  const providerId = useSignal('');
  const kaneoProjectId = useSignal(projectId ? '' : '');
  const enabled = useSignal(true);
  const pickupColumn = useSignal('');
  const progressColumn = useSignal('');
  const targetColumn = useSignal('');
  const promptTemplate = useSignal('');
  const repoUrl = useSignal('');
  const agentsMd = useSignal('');
  const submitting = useSignal(false);
  const formError = useSignal<string | null>(null);

  const lookup = useProjectFormLookup(providerId, kaneoProjectId);
  const providerForm = useProjectFormProvider((newId: string) => {
    providerId.value = newId;
    lookup.resetLookup();
  });

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProject>[0]) => createProject(input),
    {
      onSuccess: () => {
        invalidate('projects');
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProject>[1] }) =>
      updateProject(id, input),
    {
      onSuccess: () => {
        invalidate('projects');
      }
    }
  );

  useEffect(() => {
    if (projectId && existingProject) {
      const p = existingProject;
      kaneoProjectId.value = p.kaneoProjectId;
      providerId.value = p.providerId ?? '';
      enabled.value = p.enabled;
      pickupColumn.value = p.pickupColumn;
      progressColumn.value = p.progressColumn;
      targetColumn.value = p.targetColumn;
      promptTemplate.value = p.promptTemplate;
      repoUrl.value = p.repoUrl;
      agentsMd.value = p.agentsMd;
    }
  }, [projectId, existingProject]);

  useEffect(() => {
    if (projectId && existingProject && providerId.value) {
      lookup.resetLookup();
      lookupProject(providerId.value, existingProject.kaneoProjectId)
        .then((result) => {
          lookup.lookupProjectName.value = result.name;
          lookup.columns.value = result.columns;
          lookup.lookedUp.value = true;
        })
        .catch((err) => {
          lookup.lookupError.value = err instanceof Error ? err.message : 'Lookup failed';
        })
        .finally(() => {
          lookup.columnsLoading.value = false;
        });
    }
  }, [projectId, existingProject, providerId.value]);

  const handleSubmit = useCallback(
    async (e: Event) => {
      e.preventDefault();
      formError.value = null;

      if (!promptTemplate.value) {
        formError.value = 'Prompt template is required';
        return;
      }

      submitting.value = true;

      try {
        if (projectId) {
          await updateMutation.mutateAsync({
            id: projectId,
            input: {
              enabled: enabled.value,
              pickupColumn: pickupColumn.value || undefined,
              progressColumn: progressColumn.value || undefined,
              targetColumn: targetColumn.value || undefined,
              promptTemplate: promptTemplate.value || undefined,
              repoUrl: repoUrl.value || undefined,
              agentsMd: agentsMd.value || undefined,
              providerId: providerId.value || undefined
            }
          });
        } else {
          if (!providerId.value) {
            formError.value = 'Provider is required';
            submitting.value = false;
            return;
          }
          await createMutation.mutateAsync({
            kaneoProjectId: kaneoProjectId.value,
            providerId: providerId.value,
            enabled: enabled.value,
            pickupColumn: pickupColumn.value || undefined,
            progressColumn: progressColumn.value || undefined,
            targetColumn: targetColumn.value || undefined,
            promptTemplate: promptTemplate.value,
            repoUrl: repoUrl.value || undefined,
            agentsMd: agentsMd.value || undefined
          });
        }
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save project config';
      } finally {
        submitting.value = false;
      }
    },
    [projectId, createMutation, updateMutation]
  );

  const resetLookup = () => lookup.resetLookup();

  return {
    isEdit: !!projectId,
    projectLoading: projectId ? projectLoading : false,
    providers,
    providerId,
    kaneoProjectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    submitting,
    formError,
    columns: lookup.columns,
    columnsLoading: lookup.columnsLoading,
    lookupProjectName: lookup.lookupProjectName,
    lookupError: lookup.lookupError,
    lookedUp: lookup.lookedUp,
    showProviderForm: providerForm.showProviderForm,
    newProviderName: providerForm.newProviderName,
    newProviderUrl: providerForm.newProviderUrl,
    newProviderKey: providerForm.newProviderKey,
    newProviderType: providerForm.newProviderType,
    providerFormError: providerForm.providerFormError,
    providerSubmitting: providerForm.providerSubmitting,
    handleLookup: lookup.handleLookup,
    handleSubmit,
    handleCreateProvider: providerForm.handleCreateProvider,
    onShowProviderForm: providerForm.onShowProviderForm,
    onCancelProviderForm: providerForm.onCancelProviderForm,
    onProviderChange: (id: string) => {
      providerId.value = id;
      resetLookup();
    },
    onProjectIdChange: (id: string) => {
      kaneoProjectId.value = id;
      resetLookup();
    },
    onEnabledChange: (checked: boolean) => {
      enabled.value = checked;
    },
    onPromptTemplateChange: (value: string) => {
      promptTemplate.value = value;
    },
    onRepoUrlChange: (value: string) => {
      repoUrl.value = value;
    },
    onAgentsMdChange: (value: string) => {
      agentsMd.value = value;
    }
  };
};
