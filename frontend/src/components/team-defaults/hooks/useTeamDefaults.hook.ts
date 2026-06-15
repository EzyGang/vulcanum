import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useModelItems } from '../../../hooks/useModelItems.hook';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../../../services/model-providers/model-providers.service';
import { getTeam, updateTeam } from '../../../services/teams/teams.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { textInputHandler } from '../../../utils/signalInput';

export const useTeamDefaults = (teamId: string | null) => {
  const promptTemplate = useSignal('');
  const agentsMd = useSignal('');
  const primaryModelProviderKey = useSignal('');
  const primaryModelId = useSignal('');
  const smallModelProviderKey = useSignal('');
  const smallModelId = useSignal('');
  const formError = useSignal<string | null>(null);

  const { data: team, isLoading } = useApiQuery(
    ['team', teamId ?? ''],
    () => getTeam(teamId ?? ''),
    { enabled: !!teamId }
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
    promptTemplate.value = team.promptTemplate;
    agentsMd.value = team.agentsMd;
    primaryModelProviderKey.value = team.primaryModelProviderKey ?? '';
    primaryModelId.value = team.primaryModelId ?? '';
    smallModelProviderKey.value = team.smallModelProviderKey ?? '';
    smallModelId.value = team.smallModelId ?? '';
  }, [teamId, team]);

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
      connectedProviderItems,
      primaryModelItems,
      smallModelItems
    },
    status: {
      loading: isLoading,
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
            primaryModelProviderKey: primaryModelProviderKey.value || null,
            primaryModelId: primaryModelId.value || null,
            smallModelProviderKey: smallModelProviderKey.value || null,
            smallModelId: smallModelId.value || null
          });
        } catch (err) {
          formError.value = err instanceof Error ? err.message : 'Failed to update team defaults';
        }
      }
    }
  };
};
