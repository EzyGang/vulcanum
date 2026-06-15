import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { useModelItems } from '../../../../hooks/useModelItems.hook';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../../../../services/model-providers/model-providers.service';
import { getProject } from '../../../../services/projects/projects.service';
import { listProviders, lookupProject } from '../../../../services/providers/providers.service';
import { useApiQuery } from '../../../../utils/api/query/hooks';
import {
  getProjectSetupHelpText,
  getProjectSetupMissingMessages
} from '../../../../utils/projectSetup';
import { textInputHandler } from '../../../../utils/signalInput';
import { useGitHubApp } from '../../../github/hooks/useGitHubApp.hook';
import type { ProjectFormFieldsContextValue } from '../../context/ProjectFormFieldsContext';
import type { ProjectFormLookupContextValue } from '../../context/ProjectFormLookupContext';
import type { ProjectFormMetaContextValue } from '../../context/ProjectFormMetaContext';
import type { ProjectFormProviderContextValue } from '../../context/ProjectFormProviderContext';
import { DEFAULT_PROJECT_PROMPT_TEMPLATE } from '../../projectPromptTemplate';
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

  const { data: providers = [], isLoading: providersLoading } = useApiQuery(['providers'], () =>
    listProviders()
  );
  const { repos, reposLoading } = useGitHubApp();
  const { data: modelProviders = [], isLoading: modelProvidersLoading } = useApiQuery(
    ['model-providers'],
    () => listModelProviders()
  );
  const { data: modelCatalog } = useApiQuery(['model-provider-catalog'], () =>
    getModelProviderCatalog()
  );

  const providerId = useSignal('');
  const externalProjectId = useSignal(projectId ? '' : '');
  const workspaceId = useSignal('');
  const name = useSignal('');
  const enabled = useSignal(true);
  const pickupColumn = useSignal('');
  const progressColumn = useSignal('');
  const targetColumn = useSignal('');
  const promptTemplate = useSignal(DEFAULT_PROJECT_PROMPT_TEMPLATE);
  const repoFullNames = useSignal<string[]>([]);
  const agentsMd = useSignal('');
  const overridesOpen = useSignal(false);
  const primaryModelProviderKey = useSignal('');
  const primaryModelId = useSignal('');
  const smallModelProviderKey = useSignal('');
  const smallModelId = useSignal('');

  const { formError, submitting, handleSubmit } = useProjectFormSubmit({
    projectId,
    name,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoFullNames,
    agentsMd,
    overridesOpen,
    primaryModelProviderKey,
    primaryModelId,
    smallModelProviderKey,
    smallModelId,
    providerId,
    externalProjectId,
    workspaceId
  });

  const lookup = useProjectFormLookup(providerId, externalProjectId, workspaceId, submitting);
  const providerForm = useProjectFormProvider((newId: string) => {
    providerId.value = newId;
    name.value = '';
    lookup.resetLookup();
  });

  useEffect(() => {
    if (projectId && existingProject) {
      const p = existingProject;
      externalProjectId.value = p.externalProjectId;
      workspaceId.value = p.externalWorkspaceId;
      name.value = p.name || '';
      providerId.value = p.providerId ?? '';
      enabled.value = p.enabled;
      pickupColumn.value = p.pickupColumn;
      progressColumn.value = p.progressColumn;
      targetColumn.value = p.targetColumn;
      promptTemplate.value = p.promptTemplate ?? DEFAULT_PROJECT_PROMPT_TEMPLATE;
      repoFullNames.value = p.repoFullNames ?? [];
      agentsMd.value = p.agentsMd ?? '';
      overridesOpen.value = !!p.promptTemplate || !!p.agentsMd || !!p.primaryModelProviderKey;
      primaryModelProviderKey.value = p.primaryModelProviderKey ?? '';
      primaryModelId.value = p.primaryModelId ?? '';
      smallModelProviderKey.value = p.smallModelProviderKey ?? '';
      smallModelId.value = p.smallModelId ?? '';
    }
  }, [projectId, existingProject]);

  useEffect(() => {
    if (
      projectId &&
      existingProject &&
      providerId.value &&
      existingProject.externalWorkspaceId &&
      workspaceId.value === existingProject.externalWorkspaceId &&
      externalProjectId.value === existingProject.externalProjectId
    ) {
      lookup.lookupError.value = null;
      lookup.columnsLoading.value = true;
      lookup.columns.value = [];
      lookup.lookedUp.value = false;
      lookupProject(providerId.value, existingProject.externalProjectId)
        .then((result) => {
          lookup.lookupProjectName.value = result.name;
          name.value = result.name;
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
  }, [projectId, existingProject, providerId.value, workspaceId.value, externalProjectId.value]);

  useEffect(() => {
    if (providerId.value) {
      lookup.fetchWorkspaces();
    }
  }, [providerId.value]);

  useEffect(() => {
    if (projectId && providerId.value && workspaceId.value) {
      lookup.fetchProjects(workspaceId.value);
    }
  }, [projectId, providerId.value, workspaceId.value]);

  const hasProjectSelection = !!workspaceId.value && !!externalProjectId.value;
  const hasColumns = lookup.lookedUp.value && lookup.columns.value.length > 0;
  const setupWarning =
    providersLoading || modelProvidersLoading
      ? ''
      : getProjectSetupHelpText(
          getProjectSetupMissingMessages({
            hasTaskTrackerProvider: providers.length > 0,
            hasModelProvider: modelProviders.length > 0
          })
        );
  const catalogProviders = modelCatalog?.providers ?? [];
  const { connectedProviderItems, primaryModelItems, smallModelItems } = useModelItems({
    modelProviders,
    catalogProviders,
    primaryModelProviderKey,
    smallModelProviderKey
  });

  return {
    meta: {
      isEdit: !!projectId,
      projectLoading: projectId ? projectLoading : false,
      submitting,
      formError,
      canShowLookup: !!projectId || !!providerId.value,
      canShowFields: hasProjectSelection && hasColumns,
      projectSetupWarning: setupWarning,
      onSubmit: handleSubmit,
      onCancel: () => setLocation('/settings?tab=projects')
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
        name.value = '';
        lookup.resetLookup();
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
      lookedUp: lookup.lookedUp,
      workspaceOptions: lookup.workspaceOptions,
      workspaceId: lookup.workspaceId,
      projectOptions: lookup.projectOptions,
      workspacesLoading: lookup.workspacesLoading,
      projectsLoading: lookup.projectsLoading,
      workspaceSelectDisabled: lookup.workspaceSelectDisabled,
      projectSelectDisabled: lookup.projectSelectDisabled,
      onLookup: async () => {
        await lookup.handleLookup();
        name.value = lookup.lookupProjectName.value;
      },
      onProjectIdChange: (id: string) => {
        externalProjectId.value = id;
        name.value = '';
        lookup.resetLookup();
      },
      onWorkspaceChange: (id: string) => {
        name.value = '';
        lookup.handleWorkspaceChange(id);
      },
      onProjectSelectById: (id: string) => {
        lookup.handleProjectSelectById(id);
        name.value = lookup.lookupProjectName.value;
      },
      fetchProjects: lookup.fetchProjects
    },
    fields: {
      enabled,
      pickupColumn,
      progressColumn,
      targetColumn,
      columns: lookup.columns,
      columnsLoading: lookup.columnsLoading,
      promptTemplate,
      repoFullNames,
      agentsMd,
      overridesOpen,
      primaryModelProviderKey,
      primaryModelId,
      smallModelProviderKey,
      smallModelId,
      modelProviders,
      catalogProviders,
      connectedProviderItems,
      primaryModelItems,
      smallModelItems,
      repoItems: repos.map((repo) => ({
        fullName: repo,
        checked: repoFullNames.value.includes(repo),
        onCheckedChange: (checked: boolean) => {
          repoFullNames.value = checked
            ? [...repoFullNames.value, repo]
            : repoFullNames.value.filter((selectedRepo) => selectedRepo !== repo);
        }
      })),
      reposLoading,
      overridesToggleLabel: overridesOpen.value ? 'Hide' : 'Show',
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
      onPromptTemplateInput: textInputHandler(promptTemplate),
      onPromptTemplateChange: (value: string) => {
        promptTemplate.value = value;
      },
      onAgentsMdInput: textInputHandler(agentsMd),
      onAgentsMdChange: (value: string) => {
        agentsMd.value = value;
      },
      onToggleOverrides: () => {
        overridesOpen.value = !overridesOpen.value;
      },
      onPrimaryModelProviderChange: (value: string) => {
        primaryModelProviderKey.value = value;
        primaryModelId.value = '';
      },
      onPrimaryModelChange: (value: string) => {
        primaryModelId.value = value;
      },
      onSmallModelProviderChange: (value: string) => {
        smallModelProviderKey.value = value;
        smallModelId.value = '';
      },
      onSmallModelChange: (value: string) => {
        smallModelId.value = value;
      }
    }
  };
};
