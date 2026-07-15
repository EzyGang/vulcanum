import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import {
  DEFAULT_MAX_IN_PROGRESS_TASKS,
  DEFAULT_REVIEW_MAX_TURNS
} from '../../../constants/reviewAutomation';
import { updateProject } from '../../../services/projects/projects.service';
import type { ProjectConfig, UpdateProjectRequest } from '../../../types/projects';
import type { TaskBoardColumn } from '../../../types/task-board';
import { invalidate, queryClient } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';
import { parsePositiveNumber } from '../../../utils/numbers';
import { textInputHandler } from '../../../utils/signal-input';
import type { TaskBoardColumnRole } from '../types';
import {
  nullableText,
  projectConfigsQueryKey,
  settingsFormFromConfig,
  taskBoardProjectsQueryKey
} from './taskBoard.helpers';
import { useTaskBoardHelpCards } from './useTaskBoardHelpCards.hook';

interface TaskBoardSelection {
  providerId: string;
  externalProjectId: string;
}

export const useTaskBoardSettings = (
  selection: TaskBoardSelection | null,
  columns: TaskBoardColumn[],
  projectConfig: ProjectConfig | null,
  selectedRepoNames: string[]
) => {
  const formError = useSignal<string | null>(null);
  const settingsDialogOpen = useSignal(false);
  const settingsPromptTemplate = useSignal('');
  const settingsAgentsMd = useSignal('');
  const settingsReviewEnabled = useSignal('');
  const settingsReviewMaxTurns = useSignal('');
  const settingsReviewPromptTemplate = useSignal('');
  const settingsMaxInProgressTasks = useSignal('');
  const helpCards = useTaskBoardHelpCards();
  const automationOverride = useSignal<boolean | null>(null);

  useEffect(() => {
    if (!settingsDialogOpen.value) return;

    const nextSettings = settingsFormFromConfig(projectConfig);
    settingsPromptTemplate.value = nextSettings.promptTemplate;
    settingsAgentsMd.value = nextSettings.agentsMd;
    settingsReviewEnabled.value = nextSettings.reviewEnabled;
    settingsReviewMaxTurns.value = nextSettings.reviewMaxTurns;
    settingsReviewPromptTemplate.value = nextSettings.reviewPromptTemplate;
    settingsMaxInProgressTasks.value = nextSettings.maxInProgressTasks;
  }, [settingsDialogOpen.value, projectConfig?.id]);

  const repoMutation = useApiMutation(
    (nextRepoNames: string[]) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before pinning repositories');
      }

      return updateProject(projectConfig.id, { repoFullNames: nextRepoNames });
    },
    {
      onSuccess: () => {
        formError.value = null;
        invalidate(...projectConfigsQueryKey);
      }
    }
  );

  const settingsMutation = useApiMutation(
    (input: UpdateProjectRequest) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before editing board settings');
      }

      return updateProject(projectConfig.id, input);
    },
    {
      onSuccess: () => {
        formError.value = null;
        settingsDialogOpen.value = false;
        invalidate(...projectConfigsQueryKey);
        invalidate(...taskBoardProjectsQueryKey);
      }
    }
  );

  const columnRoleMutation = useApiMutation(
    (input: UpdateProjectRequest) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before editing column roles');
      }

      return updateProject(projectConfig.id, input);
    },
    {
      onSuccess: () => {
        formError.value = null;
        invalidate(...projectConfigsQueryKey);
        invalidate(...taskBoardProjectsQueryKey);
      }
    }
  );

  const automationMutation = useApiMutation(
    (enabled: boolean) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before toggling automation');
      }

      return updateProject(projectConfig.id, { enabled });
    },
    {
      onSuccess: async (updatedProject) => {
        formError.value = null;
        queryClient.setQueryData<ProjectConfig[]>(projectConfigsQueryKey, (configs) =>
          configs?.map((config) => (config.id === updatedProject.id ? updatedProject : config))
        );
        await Promise.all([
          invalidate(...projectConfigsQueryKey),
          invalidate(...taskBoardProjectsQueryKey)
        ]);
        automationOverride.value = null;
      },
      onError: () => {
        automationOverride.value = null;
      }
    }
  );

  const toggleAutomation = useCallback(() => {
    if (!projectConfig) {
      formError.value = 'Add this provider project before toggling automation';
      return;
    }

    const nextEnabled = !(automationOverride.value ?? projectConfig.enabled);
    automationOverride.value = nextEnabled;
    automationMutation.mutate(nextEnabled);
  }, [automationMutation, automationOverride, formError, projectConfig]);

  const submitSettings = useCallback(
    async (event: Event) => {
      event.preventDefault();
      formError.value = null;

      try {
        await settingsMutation.mutateAsync({
          promptTemplate: nullableText(settingsPromptTemplate.value),
          agentsMd: nullableText(settingsAgentsMd.value),
          reviewEnabled:
            settingsReviewEnabled.value === '' ? null : settingsReviewEnabled.value === 'true',
          reviewMaxTurns: settingsReviewMaxTurns.value.trim()
            ? parsePositiveNumber(settingsReviewMaxTurns.value, DEFAULT_REVIEW_MAX_TURNS)
            : null,
          reviewPromptTemplate: nullableText(settingsReviewPromptTemplate.value),
          maxInProgressTasks: settingsMaxInProgressTasks.value.trim()
            ? parsePositiveNumber(settingsMaxInProgressTasks.value, DEFAULT_MAX_IN_PROGRESS_TASKS)
            : null
        });
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save board settings';
      }
    },
    [
      formError,
      settingsMutation,
      settingsPromptTemplate,
      settingsAgentsMd,
      settingsReviewEnabled,
      settingsReviewMaxTurns,
      settingsReviewPromptTemplate,
      settingsMaxInProgressTasks
    ]
  );

  const setColumnRole = useCallback(
    (columnSlug: string | null, role: TaskBoardColumnRole) => {
      formError.value = null;
      const input: UpdateProjectRequest = {};
      if (columnSlug === null) {
        formError.value = 'Required board roles must use a provider column';
        return;
      }
      const requiredColumnSlug = columnSlug ?? undefined;

      if (role === 'pickup') {
        input.pickupColumn = requiredColumnSlug;
      } else if (role === 'progress') {
        input.progressColumn = requiredColumnSlug;
      } else if (role === 'review') {
        input.reviewColumn = requiredColumnSlug;
      } else if (role === 'done') {
        input.doneColumn = requiredColumnSlug;
      }

      columnRoleMutation.mutate(input, {
        onError: (err) => {
          formError.value = err.message;
        }
      });
    },
    [columnRoleMutation, formError]
  );

  const toggleRepo = useCallback(
    (repoFullName: string) => {
      if (!selection || !columns.length) {
        formError.value = 'Board columns must load before repositories can be connected';
        return;
      }

      const nextRepoNames = selectedRepoNames.includes(repoFullName)
        ? selectedRepoNames.filter((name) => name !== repoFullName)
        : [...selectedRepoNames, repoFullName];
      repoMutation.mutate(nextRepoNames, {
        onError: (err) => {
          formError.value = err.message;
        }
      });
    },
    [selection, columns, selectedRepoNames, repoMutation, formError]
  );

  const settingsForm = {
    promptTemplate: settingsPromptTemplate.value,
    agentsMd: settingsAgentsMd.value,
    reviewEnabled: settingsReviewEnabled.value,
    reviewMaxTurns: settingsReviewMaxTurns.value,
    reviewPromptTemplate: settingsReviewPromptTemplate.value,
    maxInProgressTasks: settingsMaxInProgressTasks.value
  };

  return {
    data: {
      settingsDialogOpen: settingsDialogOpen.value,
      automationEnabled: automationOverride.value ?? Boolean(projectConfig?.enabled),
      dismissedHelpCards: helpCards.dismissedHelpCards
    },
    form: settingsForm,
    status: {
      connectingRepos: repoMutation.isPending,
      connected: Boolean(projectConfig),
      savingSettings: settingsMutation.isPending,
      configuringColumns: columnRoleMutation.isPending,
      savingAutomation: automationMutation.isPending,
      settingsDisabled: settingsMutation.isPending || !projectConfig,
      repoControlsDisabled: repoMutation.isPending || !projectConfig
    },
    error:
      repoMutation.error?.message ??
      settingsMutation.error?.message ??
      columnRoleMutation.error?.message ??
      automationMutation.error?.message ??
      formError.value ??
      null,
    actions: {
      onToggleRepo: toggleRepo,
      onSettingsPromptInput: textInputHandler(settingsPromptTemplate),
      onSettingsAgentsInput: textInputHandler(settingsAgentsMd),
      onSettingsReviewEnabledChange: (value: string) => {
        settingsReviewEnabled.value = value;
      },
      onSettingsReviewMaxTurnsInput: textInputHandler(settingsReviewMaxTurns),
      onSettingsReviewPromptInput: textInputHandler(settingsReviewPromptTemplate),
      onSettingsMaxInProgressInput: textInputHandler(settingsMaxInProgressTasks),
      onSubmitSettings: submitSettings,
      onSetColumnRole: setColumnRole,
      onToggleAutomation: toggleAutomation,
      onDismissHelpCard: helpCards.onDismissHelpCard,
      onOpenSettings: () => {
        formError.value = null;
        settingsDialogOpen.value = true;
      },
      onCloseSettings: () => {
        settingsDialogOpen.value = false;
      },
      onSettingsDialogOpenChange: (open: boolean) => {
        if (!open) settingsDialogOpen.value = false;
      }
    }
  };
};
