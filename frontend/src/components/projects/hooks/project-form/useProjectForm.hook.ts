import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { getProject } from '../../../../services/projects/projects.service';
import { listProviders, lookupProject } from '../../../../services/providers/providers.service';
import { useApiQuery } from '../../../../utils/api/query/hooks';
import { useGitHubApp } from '../../../github/hooks/useGitHubApp.hook';
import type { ProjectFormFieldsContextValue } from '../../context/ProjectFormFieldsContext';
import type { ProjectFormLookupContextValue } from '../../context/ProjectFormLookupContext';
import type { ProjectFormMetaContextValue } from '../../context/ProjectFormMetaContext';
import type { ProjectFormProviderContextValue } from '../../context/ProjectFormProviderContext';
import { useProjectFormLookup } from './useProjectFormLookup.hook';
import { useProjectFormProvider } from './useProjectFormProvider.hook';
import { useProjectFormSubmit } from './useProjectFormSubmit.hook';

interface UseProjectFormResult {
  meta: ProjectFormMetaContextValue;
  provider: ProjectFormProviderContextValue;
  lookup: ProjectFormLookupContextValue;
  fields: ProjectFormFieldsContextValue;
}

export const useProjectForm = (projectId: string | null): UseProjectFormResult => {
  const [, setLocation] = useLocation();
  const { data: existingProject, isLoading: projectLoading } = useApiQuery(
    ['project', projectId ?? ''],
    () => getProject(projectId ?? '')
  );

  const { data: providers = [] } = useApiQuery(['providers'], () => listProviders());
  const { repos, reposLoading } = useGitHubApp();

  const providerId = useSignal('');
  const externalProjectId = useSignal(projectId ? '' : '');
  const enabled = useSignal(true);
  const pickupColumn = useSignal('');
  const progressColumn = useSignal('');
  const targetColumn = useSignal('');
  const promptTemplate = useSignal('');
  const repoUrl = useSignal('');
  const agentsMd = useSignal('');
  const opencodeConfig = useSignal('');

  const lookup = useProjectFormLookup(providerId, externalProjectId);
  const providerForm = useProjectFormProvider((newId: string) => {
    providerId.value = newId;
    lookup.resetLookup();
  });

  const { formError, submitting, handleSubmit } = useProjectFormSubmit({
    projectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    opencodeConfig,
    providerId,
    externalProjectId
  });

  useEffect(() => {
    if (projectId && existingProject) {
      const p = existingProject;
      externalProjectId.value = p.externalProjectId;
      providerId.value = p.providerId ?? '';
      enabled.value = p.enabled;
      pickupColumn.value = p.pickupColumn;
      progressColumn.value = p.progressColumn;
      targetColumn.value = p.targetColumn;
      promptTemplate.value = p.promptTemplate;
      repoUrl.value = p.repoUrl;
      agentsMd.value = p.agentsMd;
      opencodeConfig.value = p.opencodeConfig;
    }
  }, [projectId, existingProject]);

  useEffect(() => {
    if (projectId && existingProject && providerId.value) {
      lookup.resetLookup();
      lookupProject(providerId.value, existingProject.externalProjectId)
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

  const resetLookup = () => lookup.resetLookup();

  return {
    meta: {
      isEdit: !!projectId,
      projectLoading: projectId ? projectLoading : false,
      submitting,
      formError,
      canShowLookup: !!projectId || !!providerId.value,
      canShowFields: !!projectId || lookup.lookedUp.value,
      onSubmit: handleSubmit,
      onCancel: () => setLocation('/projects')
    },
    provider: {
      providers,
      providerId,
      showProviderForm: providerForm.showProviderForm,
      newProviderName: providerForm.newProviderName,
      newProviderUrl: providerForm.newProviderUrl,
      newProviderKey: providerForm.newProviderKey,
      newProviderType: providerForm.newProviderType,
      providerFormError: providerForm.providerFormError,
      providerSubmitting: providerForm.providerSubmitting,
      onProviderChange: (id: string) => {
        providerId.value = id;
        resetLookup();
      },
      onShowProviderForm: providerForm.onShowProviderForm,
      onCancelProviderForm: providerForm.onCancelProviderForm,
      onCreateProvider: providerForm.handleCreateProvider,
      onNewProviderNameChange: (value: string) => {
        providerForm.newProviderName.value = value;
      },
      onNewProviderUrlChange: (value: string) => {
        providerForm.newProviderUrl.value = value;
      },
      onNewProviderKeyChange: (value: string) => {
        providerForm.newProviderKey.value = value;
      },
      onNewProviderTypeChange: (value: string) => {
        providerForm.newProviderType.value = value;
      }
    },
    lookup: {
      externalProjectId,
      lookupProjectName: lookup.lookupProjectName,
      lookupError: lookup.lookupError,
      onLookup: () => {
        lookup.handleLookup();
      },
      onProjectIdChange: (id: string) => {
        externalProjectId.value = id;
        resetLookup();
      }
    },
    fields: {
      enabled,
      pickupColumn,
      progressColumn,
      targetColumn,
      columns: lookup.columns,
      columnsLoading: lookup.columnsLoading,
      promptTemplate,
      repoUrl,
      agentsMd,
      opencodeConfig,
      repos,
      reposLoading,
      onEnabledChange: (checked: boolean) => {
        enabled.value = checked;
      },
      onPickupColumnChange: (value: string) => {
        pickupColumn.value = value;
      },
      onProgressColumnChange: (value: string) => {
        progressColumn.value = value;
      },
      onTargetColumnChange: (value: string) => {
        targetColumn.value = value;
      },
      onPromptTemplateChange: (value: string) => {
        promptTemplate.value = value;
      },
      onRepoUrlChange: (value: string) => {
        repoUrl.value = value;
      },
      onAgentsMdChange: (value: string) => {
        agentsMd.value = value;
      },
      onOpencodeConfigChange: (value: string) => {
        opencodeConfig.value = value;
      }
    }
  };
};
