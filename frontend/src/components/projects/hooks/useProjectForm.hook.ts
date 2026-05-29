import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  createProject,
  getProject,
  updateProject
} from '../../../services/projects/projects.service';
import {
  createProvider,
  listProviders,
  lookupProject
} from '../../../services/providers/providers.service';
import type { ColumnInfo } from '../../../types/projects';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useProjectForm = (projectId: string | null) => {
  const [_, setLocation] = useLocation();

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
  const columns = useSignal<ColumnInfo[]>([]);
  const columnsLoading = useSignal(false);
  const lookupProjectName = useSignal('');
  const lookupError = useSignal<string | null>(null);
  const lookedUp = useSignal(false);

  const showProviderForm = useSignal(false);
  const newProviderName = useSignal('');
  const newProviderUrl = useSignal('');
  const newProviderKey = useSignal('');
  const providerFormError = useSignal<string | null>(null);
  const providerSubmitting = useSignal(false);

  const refetchProviders = () => invalidate('providers');

  const handleCreateProvider = useCallback(async (e: Event) => {
    e.preventDefault();
    providerFormError.value = null;

    if (!newProviderName.value || !newProviderUrl.value || !newProviderKey.value) {
      providerFormError.value = 'All fields are required';
      return;
    }

    providerSubmitting.value = true;
    try {
      const created = await createProvider({
        name: newProviderName.value,
        instanceUrl: newProviderUrl.value,
        apiKey: newProviderKey.value
      });
      refetchProviders();
      providerId.value = created.id;
      showProviderForm.value = false;
      newProviderName.value = '';
      newProviderUrl.value = '';
      newProviderKey.value = '';
      lookedUp.value = false;
      columns.value = [];
      lookupProjectName.value = '';
      lookupError.value = null;
      pickupColumn.value = '';
      progressColumn.value = '';
      targetColumn.value = '';
    } catch (err) {
      providerFormError.value = err instanceof Error ? err.message : 'Failed to create provider';
    } finally {
      providerSubmitting.value = false;
    }
  }, []);

  const handleLookup = useCallback(async () => {
    if (!providerId.value || !kaneoProjectId.value) return;

    lookupError.value = null;
    columnsLoading.value = true;
    lookedUp.value = false;

    try {
      const result = await lookupProject(providerId.value, kaneoProjectId.value);
      lookupProjectName.value = result.name;
      columns.value = result.columns;
      lookedUp.value = true;
    } catch (err) {
      lookupError.value = err instanceof Error ? err.message : 'Lookup failed';
      columns.value = [];
      lookupProjectName.value = '';
    } finally {
      columnsLoading.value = false;
    }
  }, []);

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
      lookupError.value = null;
      columnsLoading.value = true;
      lookedUp.value = false;
      lookupProject(providerId.value, existingProject.kaneoProjectId)
        .then((result) => {
          lookupProjectName.value = result.name;
          columns.value = result.columns;
          lookedUp.value = true;
        })
        .catch((err) => {
          lookupError.value = err instanceof Error ? err.message : 'Lookup failed';
        })
        .finally(() => {
          columnsLoading.value = false;
        });
    }
  }, [projectId, existingProject, providerId.value]);

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProject>[0]) => createProject(input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProject>[1] }) =>
      updateProject(id, input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
      }
    }
  );

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
    columns,
    columnsLoading,
    lookupProjectName,
    lookupError,
    lookedUp,
    showProviderForm,
    newProviderName,
    newProviderUrl,
    newProviderKey,
    providerFormError,
    providerSubmitting,
    handleLookup,
    handleSubmit,
    handleCreateProvider,
    onShowProviderForm: () => {
      showProviderForm.value = true;
    },
    onCancelProviderForm: () => {
      showProviderForm.value = false;
      providerFormError.value = null;
    },
    onProviderChange: (id: string) => {
      providerId.value = id;
      lookedUp.value = false;
      columns.value = [];
      lookupProjectName.value = '';
      lookupError.value = null;
      pickupColumn.value = '';
      progressColumn.value = '';
      targetColumn.value = '';
    },
    onProjectIdChange: (id: string) => {
      kaneoProjectId.value = id;
      lookedUp.value = false;
      columns.value = [];
      lookupProjectName.value = '';
      lookupError.value = null;
      pickupColumn.value = '';
      progressColumn.value = '';
      targetColumn.value = '';
    }
  };
};
