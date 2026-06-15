import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { getAuthMode } from '../../../services/auth/auth.service';
import {
  createTeamInvite,
  deleteTeam,
  getTeam,
  listTeamMembers,
  updateTeam
} from '../../../services/teams/teams.service';
import { selectedTeamId, setSelectedTeamId } from '../../../stores/auth.store';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { formatDateTime } from '../../../utils/format';

export const useTeamDetail = (teamId: string) => {
  const [, setLocation] = useLocation();
  const editName = useSignal('');
  const editing = useSignal(false);
  const formError = useSignal<string | null>(null);
  const inviteLink = useSignal<string | null>(null);
  const inviteExpiresAt = useSignal<string | null>(null);

  const { data: authMode } = useApiQuery(['auth-mode'], getAuthMode);
  const {
    data: team,
    isLoading: teamLoading,
    error: teamError
  } = useApiQuery(['team', teamId], () => getTeam(teamId));
  const { data: members = [], isLoading: membersLoading } = useApiQuery(
    ['teams', teamId, 'members'],
    () => listTeamMembers(teamId)
  );

  useEffect(() => {
    if (team && !editing.value) {
      editName.value = team.name;
    }
  }, [team, editing.value]);

  const refreshTeam = useCallback(() => {
    invalidate('team', teamId);
    invalidate('teams');
  }, [teamId]);

  const updateMutation = useApiMutation(
    (input: Parameters<typeof updateTeam>[1]) => updateTeam(teamId, input),
    { onSuccess: refreshTeam }
  );
  const deleteMutation = useApiMutation(() => deleteTeam(teamId), {
    onSuccess: () => {
      invalidate('teams');
      setLocation('/teams');
    }
  });
  const inviteMutation = useApiMutation(() => createTeamInvite(teamId));

  const handleStartEdit = useCallback(() => {
    editing.value = true;
    formError.value = null;
  }, []);

  const handleCancelEdit = useCallback(() => {
    editing.value = false;
    editName.value = team?.name ?? '';
    formError.value = null;
  }, [team]);

  const handleEditNameInput = useCallback((event: Event) => {
    editName.value = (event.target as HTMLInputElement).value;
    formError.value = null;
  }, []);

  const handleUpdate = useCallback(
    async (event: Event) => {
      event.preventDefault();
      formError.value = null;

      try {
        await updateMutation.mutateAsync({ name: editName.value });
        editing.value = false;
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to update team';
      }
    },
    [updateMutation]
  );

  const handleUseTeam = useCallback(() => {
    setSelectedTeamId(teamId);
    invalidate();
  }, [teamId]);

  const handleDelete = useCallback(async () => {
    formError.value = null;
    try {
      await deleteMutation.mutateAsync(undefined);
    } catch (err) {
      formError.value = err instanceof Error ? err.message : 'Failed to delete team';
    }
  }, [deleteMutation]);

  const handleCreateInvite = useCallback(async () => {
    formError.value = null;
    inviteLink.value = null;
    inviteExpiresAt.value = null;

    try {
      const invite = await inviteMutation.mutateAsync(undefined);
      inviteLink.value = `${window.location.origin}/invites/${invite.token}`;
      inviteExpiresAt.value = invite.expiresAt;
    } catch (err) {
      formError.value = err instanceof Error ? err.message : 'Failed to create invite';
    }
  }, [inviteMutation]);

  return {
    data: {
      team: team ? { ...team, formattedCreatedAt: formatDateTime(team.createdAt) } : null,
      members: members.map((member) => ({
        ...member,
        formattedCreatedAt: formatDateTime(member.createdAt)
      })),
      selectedTeamId: selectedTeamId.value,
      editName: editName.value,
      editing: editing.value,
      isSingleUser: authMode?.isSingleUser ?? false,
      inviteLink: inviteLink.value,
      inviteExpiresAt: inviteExpiresAt.value ? formatDateTime(inviteExpiresAt.value) : null
    },
    status: {
      loading: teamLoading,
      membersLoading,
      error: teamError,
      formError: formError.value,
      updating: updateMutation.isPending,
      deleting: deleteMutation.isPending,
      creatingInvite: inviteMutation.isPending
    },
    actions: {
      onBack: () => setLocation('/teams'),
      onUseTeam: handleUseTeam,
      onStartEdit: handleStartEdit,
      onCancelEdit: handleCancelEdit,
      onEditNameInput: handleEditNameInput,
      onUpdate: handleUpdate,
      onDelete: handleDelete,
      onCreateInvite: handleCreateInvite
    }
  };
};
