import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import {
  DEFAULT_MAX_IN_PROGRESS_TASKS,
  DEFAULT_REVIEW_MAX_TURNS,
  DEFAULT_REVIEW_PICKUP_COLUMN
} from '../../../constants/reviewAutomation';
import { useModelProviderSelection } from '../../../hooks/useModelProviderSelection.hook';
import { getTeam, getTeamDefaults, updateTeam } from '../../../services/teams/teams.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { parsePositiveNumber } from '../../../utils/numbers';
import { textInputHandler } from '../../../utils/signalInput';

export const useTeamDefaults = (teamId: string | null) => {
  const promptTemplate = useSignal('');
  const agentsMd = useSignal('');
  const reviewEnabled = useSignal(false);
  const reviewPickupColumn = useSignal(DEFAULT_REVIEW_PICKUP_COLUMN);
  const reviewMaxTurns = useSignal(DEFAULT_REVIEW_MAX_TURNS);
  const reviewPromptTemplate = useSignal('');
  const maxInProgressTasks = useSignal(DEFAULT_MAX_IN_PROGRESS_TASKS);
  const formError = useSignal<string | null>(null);
  const modelSelection = useModelProviderSelection();
  const { primaryModelProviderKey, primaryModelId, smallModelProviderKey, smallModelId } =
    modelSelection;

  const { data: team, isLoading } = useApiQuery(
    ['team', teamId ?? ''],
    () => getTeam(teamId ?? ''),
    { enabled: !!teamId }
  );
  const { data: teamDefaults, isLoading: defaultsLoading } = useApiQuery(
    ['team-defaults'],
    getTeamDefaults
  );
  useEffect(() => {
    if (!team) {
      return;
    }
    if (
      modelSelection.modelProvidersLoading &&
      modelSelection.needsLegacyModelProviderResolution(team)
    ) {
      return;
    }
    promptTemplate.value = team.promptTemplate;
    agentsMd.value = team.agentsMd;
    primaryModelProviderKey.value = modelSelection.modelProviderConfigIdForLegacyKey(
      team.primaryModelProviderConfigId,
      team.primaryModelProviderKey
    );
    primaryModelId.value = team.primaryModelId ?? '';
    smallModelProviderKey.value = modelSelection.modelProviderConfigIdForLegacyKey(
      team.smallModelProviderConfigId,
      team.smallModelProviderKey
    );
    smallModelId.value = team.smallModelId ?? '';
    reviewEnabled.value = team.reviewEnabled;
    reviewPickupColumn.value = team.reviewPickupColumn;
    reviewMaxTurns.value = team.reviewMaxTurns;
    reviewPromptTemplate.value = reviewPromptTemplateOrDefault(
      team.reviewPromptTemplate,
      teamDefaults?.reviewPromptTemplate ?? ''
    );
    maxInProgressTasks.value = team.maxInProgressTasks;
  }, [
    teamId,
    team,
    teamDefaults,
    modelSelection.modelProviders,
    modelSelection.modelProvidersLoading
  ]);

  const mutation = useApiMutation(
    (input: Parameters<typeof updateTeam>[1]) => updateTeam(teamId ?? '', input),
    {
      onSuccess: () => {
        invalidate('team', teamId ?? '');
        invalidate('teams');
        invalidate('projects');
      }
    }
  );

  return {
    data: {
      promptTemplate,
      agentsMd,
      primaryModelProviderKey,
      primaryModelId,
      smallModelProviderKey,
      smallModelId,
      reviewEnabled,
      reviewPickupColumn,
      reviewMaxTurns,
      reviewPromptTemplate,
      maxInProgressTasks,
      connectedProviderItems: modelSelection.connectedProviderItems,
      primaryModelItems: modelSelection.primaryModelItems,
      smallModelItems: modelSelection.smallModelItems
    },
    status: {
      loading: isLoading || defaultsLoading || modelSelection.modelProvidersLoading,
      saving: mutation.isPending,
      error: formError
    },
    actions: {
      onPromptTemplateInput: textInputHandler(promptTemplate),
      onAgentsMdInput: textInputHandler(agentsMd),
      onPrimaryProviderChange: modelSelection.onPrimaryProviderChange,
      onPrimaryModelChange: modelSelection.onPrimaryModelChange,
      onSmallProviderChange: modelSelection.onSmallProviderChange,
      onSmallModelChange: modelSelection.onSmallModelChange,
      onReviewEnabledChange: (checked: boolean) => {
        reviewEnabled.value = checked;
      },
      onReviewPickupColumnInput: textInputHandler(reviewPickupColumn),
      onReviewMaxTurnsInput: (event: Event) => {
        reviewMaxTurns.value = parsePositiveNumber(
          (event.target as HTMLInputElement).value,
          DEFAULT_REVIEW_MAX_TURNS
        );
      },
      onReviewPromptTemplateInput: textInputHandler(reviewPromptTemplate),
      onMaxInProgressTasksInput: (event: Event) => {
        maxInProgressTasks.value = parsePositiveNumber(
          (event.target as HTMLInputElement).value,
          DEFAULT_MAX_IN_PROGRESS_TASKS
        );
      },
      onSubmit: async (event: Event) => {
        event.preventDefault();
        if (!teamId) {
          formError.value = 'Select a team first';
          return;
        }
        formError.value = null;
        try {
          await mutation.mutateAsync({
            promptTemplate: promptTemplate.value,
            agentsMd: agentsMd.value,
            primaryModelProviderConfigId: primaryModelProviderKey.value || null,
            primaryModelId: primaryModelId.value || null,
            smallModelProviderConfigId: smallModelProviderKey.value || null,
            smallModelId: smallModelId.value || null,
            reviewEnabled: reviewEnabled.value,
            reviewPickupColumn: reviewPickupColumn.value || DEFAULT_REVIEW_PICKUP_COLUMN,
            reviewMaxTurns: reviewMaxTurns.value,
            reviewPromptTemplate: reviewPromptTemplateForSubmit(
              team?.reviewPromptTemplate,
              reviewPromptTemplate.value,
              teamDefaults?.reviewPromptTemplate
            ),
            maxInProgressTasks: maxInProgressTasks.value
          });
        } catch (err) {
          formError.value = err instanceof Error ? err.message : 'Failed to update team defaults';
        }
      }
    }
  };
};

const reviewPromptTemplateOrDefault = (template: string, defaultTemplate: string): string => {
  if (template.trim()) {
    return template;
  }

  return defaultTemplate;
};

const reviewPromptTemplateForSubmit = (
  storedTemplate: string | undefined,
  formTemplate: string,
  defaultTemplate: string | undefined
): string => {
  if (
    !storedTemplate?.trim() &&
    defaultTemplate !== undefined &&
    formTemplate === defaultTemplate
  ) {
    return '';
  }

  return formTemplate;
};
