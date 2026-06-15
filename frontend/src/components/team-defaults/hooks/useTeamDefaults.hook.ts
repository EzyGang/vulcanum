import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { getTeam, updateTeam } from '../../../services/teams/teams.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useTeamDefaults = (teamId: string | null) => {
  const promptTemplate = useSignal('');
  const agentsMd = useSignal('');
  const primaryModelProviderKey = useSignal('');
  const primaryModelId = useSignal('');
  const smallModelProviderKey = useSignal('');
  const smallModelId = useSignal('');
  const formError = useSignal<string | null>(null);

  const { data: team, isLoading } = useApiQuery(['team', teamId ?? ''], () =>
    getTeam(teamId ?? '')
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
  }, [team]);

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
      smallModelId
    },
    status: {
      loading: isLoading,
      saving: mutation.isPending,
      error: formError.value
    },
    actions: {
      onPromptTemplateInput: (event: Event) => {
        promptTemplate.value = (event.target as HTMLTextAreaElement).value;
      },
      onAgentsMdInput: (event: Event) => {
        agentsMd.value = (event.target as HTMLTextAreaElement).value;
      },
      onPrimaryProviderInput: (event: Event) => {
        primaryModelProviderKey.value = (event.target as HTMLInputElement).value;
      },
      onPrimaryModelInput: (event: Event) => {
        primaryModelId.value = (event.target as HTMLInputElement).value;
      },
      onSmallProviderInput: (event: Event) => {
        smallModelProviderKey.value = (event.target as HTMLInputElement).value;
      },
      onSmallModelInput: (event: Event) => {
        smallModelId.value = (event.target as HTMLInputElement).value;
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
