import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import {
  DEFAULT_MAX_IN_PROGRESS_TASKS,
  DEFAULT_REVIEW_MAX_TURNS,
  DEFAULT_REVIEW_PICKUP_COLUMN
} from '../../../constants/reviewAutomation';
import { useModelItems } from '../../../hooks/useModelItems.hook';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../../../services/model-providers/model-providers.service';
import { getTeam, getTeamDefaults, updateTeam } from '../../../services/teams/teams.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { parsePositiveNumber } from '../../../utils/numbers';
import { textInputHandler } from '../../../utils/signal-input';

export const useTeamDefaults = (teamId: string | null) => {
  const promptTemplate = useSignal('');
  const agentsMd = useSignal('');
  const primaryModelProviderKey = useSignal('');
  const primaryModelId = useSignal('');
  const smallModelProviderKey = useSignal('');
  const smallModelId = useSignal('');
  const reviewEnabled = useSignal(false);
  const reviewPickupColumn = useSignal(DEFAULT_REVIEW_PICKUP_COLUMN);
  const reviewMaxTurns = useSignal(DEFAULT_REVIEW_MAX_TURNS);
  const reviewPromptTemplate = useSignal('');
  const maxInProgressTasks = useSignal(DEFAULT_MAX_IN_PROGRESS_TASKS);
  const formError = useSignal<string | null>(null);

  const { data: team, isLoading } = useApiQuery(
    ['team', teamId ?? ''],
    () => getTeam(teamId ?? ''),
    { enabled: !!teamId }
  );
  const { data: teamDefaults, isLoading: defaultsLoading } = useApiQuery(
    ['team-defaults'],
    getTeamDefaults
  );
  const { data: modelProviders = [] } = useApiQuery(['model-providers'], () =>
    listModelProviders()
  );
  const { data: modelCatalog } = useApiQuery(['model-provider-catalog'], () =>
    getModelProviderCatalog()
  );

  useEffect(() => {
    if (!team) {
      return;
    }
    promptTemplate.value = promptTemplateOrDefault(
      team.promptTemplate,
      teamDefaults?.promptTemplate ?? ''
    );
    agentsMd.value = team.agentsMd;
    primaryModelProviderKey.value = team.primaryModelProviderKey ?? '';
    primaryModelId.value = team.primaryModelId ?? '';
    smallModelProviderKey.value = team.smallModelProviderKey ?? '';
    smallModelId.value = team.smallModelId ?? '';
    reviewEnabled.value = team.reviewEnabled;
    reviewPickupColumn.value = team.reviewPickupColumn;
    reviewMaxTurns.value = team.reviewMaxTurns;
    reviewPromptTemplate.value = promptTemplateOrDefault(
      team.reviewPromptTemplate,
      teamDefaults?.reviewPromptTemplate ?? ''
    );
    maxInProgressTasks.value = team.maxInProgressTasks;
  }, [teamId, team, teamDefaults]);

  const catalogProviders = modelCatalog?.providers ?? [];
  const { connectedProviderItems, primaryModelItems, smallModelItems } = useModelItems({
    modelProviders,
    catalogProviders,
    primaryModelProviderKey,
    smallModelProviderKey
  });

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
      connectedProviderItems,
      primaryModelItems,
      smallModelItems
    },
    status: {
      loading: isLoading || defaultsLoading,
      saving: mutation.isPending,
      error: formError
    },
    actions: {
      onPromptTemplateInput: textInputHandler(promptTemplate),
      onAgentsMdInput: textInputHandler(agentsMd),
      onPrimaryProviderChange: (value: string) => {
        primaryModelProviderKey.value = value;
        primaryModelId.value = '';
      },
      onPrimaryModelChange: (value: string) => {
        primaryModelId.value = value;
      },
      onSmallProviderChange: (value: string) => {
        smallModelProviderKey.value = value;
        smallModelId.value = '';
      },
      onSmallModelChange: (value: string) => {
        smallModelId.value = value;
      },
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
            promptTemplate: promptTemplateForSubmit(
              team?.promptTemplate,
              promptTemplate.value,
              teamDefaults?.promptTemplate
            ),
            agentsMd: agentsMd.value,
            primaryModelProviderKey: primaryModelProviderKey.value || null,
            primaryModelId: primaryModelId.value || null,
            smallModelProviderKey: smallModelProviderKey.value || null,
            smallModelId: smallModelId.value || null,
            reviewEnabled: reviewEnabled.value,
            reviewPickupColumn: reviewPickupColumn.value || DEFAULT_REVIEW_PICKUP_COLUMN,
            reviewMaxTurns: reviewMaxTurns.value,
            reviewPromptTemplate: promptTemplateForSubmit(
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

const promptTemplateOrDefault = (template: string, defaultTemplate: string): string => {
  if (template.trim()) {
    return template;
  }

  return defaultTemplate;
};

const promptTemplateForSubmit = (
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
