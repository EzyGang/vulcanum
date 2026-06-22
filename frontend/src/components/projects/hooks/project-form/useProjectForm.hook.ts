import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  DEFAULT_MAX_IN_PROGRESS_TASKS,
  DEFAULT_REVIEW_MAX_TURNS,
  DEFAULT_REVIEW_PICKUP_COLUMN
} from '../../../../constants/reviewAutomation';
import { useModelItems } from '../../../../hooks/useModelItems.hook';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../../../../services/model-providers/model-providers.service';
import { getProject } from '../../../../services/projects/projects.service';
import { listProviders, lookupProject } from '../../../../services/providers/providers.service';
import { useApiQuery } from '../../../../utils/api/query/hooks';
import { parsePositiveNumber } from '../../../../utils/numbers';
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
  const promptTemplateOverride = useSignal(false);
  const repoFullNames = useSignal<string[]>([]);
  const agentsMd = useSignal('');
  const agentsMdOverride = useSignal(false);
  const overridesOpen = useSignal(false);
  const primaryModelProviderKey = useSignal('');
  const primaryModelProviderOverride = useSignal(false);
  const primaryModelId = useSignal('');
  const primaryModelIdOverride = useSignal(false);
  const smallModelProviderKey = useSignal('');
  const smallModelProviderOverride = useSignal(false);
  const smallModelId = useSignal('');
  const smallModelIdOverride = useSignal(false);
  const reviewEnabled = useSignal(false);
  const reviewEnabledOverride = useSignal(false);
  const reviewPickupColumn = useSignal(DEFAULT_REVIEW_PICKUP_COLUMN);
  const reviewPickupColumnOverride = useSignal(false);
  const reviewMaxTurns = useSignal(DEFAULT_REVIEW_MAX_TURNS);
  const reviewMaxTurnsOverride = useSignal(false);
  const reviewPromptTemplate = useSignal('');
  const reviewPromptTemplateOverride = useSignal(false);
  const maxInProgressTasks = useSignal(DEFAULT_MAX_IN_PROGRESS_TASKS);
  const maxInProgressTasksOverride = useSignal(false);

  const { formError, submitting, handleSubmit } = useProjectFormSubmit({
    projectId,
    name,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    promptTemplateOverride,
    repoFullNames,
    agentsMd,
    agentsMdOverride,
    primaryModelProviderKey,
    primaryModelProviderOverride,
    primaryModelId,
    primaryModelIdOverride,
    smallModelProviderKey,
    smallModelProviderOverride,
    smallModelId,
    smallModelIdOverride,
    reviewEnabled,
    reviewEnabledOverride,
    reviewPickupColumn,
    reviewPickupColumnOverride,
    reviewMaxTurns,
    reviewMaxTurnsOverride,
    reviewPromptTemplate,
    reviewPromptTemplateOverride,
    maxInProgressTasks,
    maxInProgressTasksOverride,
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
      promptTemplateOverride.value = p.promptTemplate != null;
      repoFullNames.value = p.repoFullNames ?? [];
      agentsMd.value = p.agentsMd ?? '';
      agentsMdOverride.value = p.agentsMd != null;
      overridesOpen.value = false;
      primaryModelProviderKey.value = p.primaryModelProviderConfigId ?? '';
      primaryModelProviderOverride.value = p.primaryModelProviderConfigId != null;
      primaryModelId.value = p.primaryModelId ?? '';
      primaryModelIdOverride.value = p.primaryModelId != null;
      smallModelProviderKey.value = p.smallModelProviderConfigId ?? '';
      smallModelProviderOverride.value = p.smallModelProviderConfigId != null;
      smallModelId.value = p.smallModelId ?? '';
      smallModelIdOverride.value = p.smallModelId != null;
      reviewEnabled.value = p.reviewEnabled ?? false;
      reviewEnabledOverride.value = p.reviewEnabled != null;
      reviewPickupColumn.value = p.reviewPickupColumn ?? DEFAULT_REVIEW_PICKUP_COLUMN;
      reviewPickupColumnOverride.value = p.reviewPickupColumn != null;
      reviewMaxTurns.value = p.reviewMaxTurns ?? DEFAULT_REVIEW_MAX_TURNS;
      reviewMaxTurnsOverride.value = p.reviewMaxTurns != null;
      reviewPromptTemplate.value = p.reviewPromptTemplate ?? '';
      reviewPromptTemplateOverride.value = p.reviewPromptTemplate != null;
      maxInProgressTasks.value = p.maxInProgressTasks ?? DEFAULT_MAX_IN_PROGRESS_TASKS;
      maxInProgressTasksOverride.value = p.maxInProgressTasks != null;
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
  const hasOverrides =
    promptTemplateOverride.value ||
    agentsMdOverride.value ||
    primaryModelProviderOverride.value ||
    primaryModelIdOverride.value ||
    smallModelProviderOverride.value ||
    smallModelIdOverride.value ||
    reviewEnabledOverride.value ||
    reviewPickupColumnOverride.value ||
    reviewMaxTurnsOverride.value ||
    reviewPromptTemplateOverride.value ||
    maxInProgressTasksOverride.value;

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
      promptTemplateOverride,
      repoFullNames,
      agentsMd,
      agentsMdOverride,
      overridesOpen,
      primaryModelProviderKey,
      primaryModelProviderOverride,
      primaryModelId,
      primaryModelIdOverride,
      smallModelProviderKey,
      smallModelProviderOverride,
      smallModelId,
      smallModelIdOverride,
      reviewEnabled,
      reviewEnabledOverride,
      reviewPickupColumn,
      reviewPickupColumnOverride,
      reviewMaxTurns,
      reviewMaxTurnsOverride,
      reviewPromptTemplate,
      reviewPromptTemplateOverride,
      maxInProgressTasks,
      maxInProgressTasksOverride,
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
      hasOverrides,
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
      onPromptTemplateInput: (event: Event) => {
        promptTemplateOverride.value = true;
        textInputHandler(promptTemplate)(event);
      },
      onPromptTemplateChange: (value: string) => {
        promptTemplateOverride.value = true;
        promptTemplate.value = value;
      },
      onResetPromptTemplateOverride: () => {
        promptTemplateOverride.value = false;
        promptTemplate.value = DEFAULT_PROJECT_PROMPT_TEMPLATE;
      },
      onAgentsMdInput: (event: Event) => {
        agentsMdOverride.value = true;
        textInputHandler(agentsMd)(event);
      },
      onAgentsMdChange: (value: string) => {
        agentsMdOverride.value = true;
        agentsMd.value = value;
      },
      onResetAgentsMdOverride: () => {
        agentsMdOverride.value = false;
        agentsMd.value = '';
      },
      onToggleOverrides: () => {
        overridesOpen.value = !overridesOpen.value;
      },
      onPrimaryModelProviderChange: (value: string) => {
        primaryModelProviderOverride.value = true;
        primaryModelProviderKey.value = value;
        primaryModelIdOverride.value = false;
        primaryModelId.value = '';
      },
      onResetPrimaryModelProviderOverride: () => {
        primaryModelProviderOverride.value = false;
        primaryModelProviderKey.value = '';
        primaryModelIdOverride.value = false;
        primaryModelId.value = '';
      },
      onPrimaryModelChange: (value: string) => {
        primaryModelIdOverride.value = true;
        primaryModelId.value = value;
      },
      onResetPrimaryModelOverride: () => {
        primaryModelIdOverride.value = false;
        primaryModelId.value = '';
      },
      onSmallModelProviderChange: (value: string) => {
        smallModelProviderOverride.value = true;
        smallModelProviderKey.value = value;
        smallModelIdOverride.value = false;
        smallModelId.value = '';
      },
      onResetSmallModelProviderOverride: () => {
        smallModelProviderOverride.value = false;
        smallModelProviderKey.value = '';
        smallModelIdOverride.value = false;
        smallModelId.value = '';
      },
      onSmallModelChange: (value: string) => {
        smallModelIdOverride.value = true;
        smallModelId.value = value;
      },
      onResetSmallModelOverride: () => {
        smallModelIdOverride.value = false;
        smallModelId.value = '';
      },
      onReviewEnabledChange: (checked: boolean) => {
        reviewEnabledOverride.value = true;
        reviewEnabled.value = checked;
      },
      onResetReviewOverrides: () => {
        reviewEnabledOverride.value = false;
        reviewEnabled.value = false;
        reviewPickupColumnOverride.value = false;
        reviewPickupColumn.value = DEFAULT_REVIEW_PICKUP_COLUMN;
        reviewMaxTurnsOverride.value = false;
        reviewMaxTurns.value = DEFAULT_REVIEW_MAX_TURNS;
        reviewPromptTemplateOverride.value = false;
        reviewPromptTemplate.value = '';
        maxInProgressTasksOverride.value = false;
        maxInProgressTasks.value = DEFAULT_MAX_IN_PROGRESS_TASKS;
      },
      onResetReviewEnabledOverride: () => {
        reviewEnabledOverride.value = false;
        reviewEnabled.value = false;
      },
      onReviewPickupColumnChange: (value: string) => {
        reviewPickupColumnOverride.value = true;
        reviewPickupColumn.value = value;
      },
      onResetReviewPickupColumnOverride: () => {
        reviewPickupColumnOverride.value = false;
        reviewPickupColumn.value = DEFAULT_REVIEW_PICKUP_COLUMN;
      },
      onReviewMaxTurnsInput: (event: Event) => {
        reviewMaxTurnsOverride.value = true;
        reviewMaxTurns.value = parsePositiveNumber(
          (event.target as HTMLInputElement).value,
          DEFAULT_REVIEW_MAX_TURNS
        );
      },
      onResetReviewMaxTurnsOverride: () => {
        reviewMaxTurnsOverride.value = false;
        reviewMaxTurns.value = DEFAULT_REVIEW_MAX_TURNS;
      },
      onReviewPromptTemplateInput: (event: Event) => {
        reviewPromptTemplateOverride.value = true;
        textInputHandler(reviewPromptTemplate)(event);
      },
      onResetReviewPromptTemplateOverride: () => {
        reviewPromptTemplateOverride.value = false;
        reviewPromptTemplate.value = '';
      },
      onMaxInProgressTasksInput: (event: Event) => {
        maxInProgressTasksOverride.value = true;
        maxInProgressTasks.value = parsePositiveNumber(
          (event.target as HTMLInputElement).value,
          DEFAULT_MAX_IN_PROGRESS_TASKS
        );
      },
      onResetMaxInProgressTasksOverride: () => {
        maxInProgressTasksOverride.value = false;
        maxInProgressTasks.value = DEFAULT_MAX_IN_PROGRESS_TASKS;
      }
    }
  };
};
