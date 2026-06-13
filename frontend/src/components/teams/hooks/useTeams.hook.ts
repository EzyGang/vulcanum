import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { getAuthMode } from '../../../services/auth/auth.service';
import {
  createTeam,
  createTeamInvite,
  deleteTeam,
  listTeamMembers,
  listTeams,
  updateTeam
} from '../../../services/teams/teams.service';
import {
  selectedTeamId,
  setSelectedTeamId,
  teams as storedTeams
} from '../../../stores/auth.store';
import type { Team } from '../../../types/teams';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useTeams = () => {
  const selectedManageTeamId = useSignal<string | null>(selectedTeamId.value);
  const name = useSignal('');
  const editName = useSignal('');
  const editingTeamId = useSignal<string | null>(null);
  const formError = useSignal<string | null>(null);
  const inviteLink = useSignal<string | null>(null);
  const inviteExpiresAt = useSignal<string | null>(null);

  const { data: authMode } = useApiQuery(['auth-mode'], getAuthMode);
  const { data: teamList = [], isLoading: loading, error } = useApiQuery(['teams'], listTeams);

  useEffect(() => {
    storedTeams.value = teamList.map((team) => ({ id: team.id, name: team.name }));
    const selectedStillExists = teamList.some((team) => team.id === selectedManageTeamId.value);
    if (!selectedStillExists && teamList[0]) {
      selectedManageTeamId.value = teamList[0].id;
    }
  }, [teamList, selectedManageTeamId.value]);

  const selectedTeam = teamList.find((team) => team.id === selectedManageTeamId.value) ?? null;
  const membersQuery = useApiQuery(
    ['teams', selectedManageTeamId.value, 'members'],
    () => listTeamMembers(selectedManageTeamId.value ?? ''),
    { enabled: Boolean(selectedManageTeamId.value) }
  );

  const refreshTeams = useCallback(() => {
    invalidate('teams');
  }, []);

  const createMutation = useApiMutation(
    (input: Parameters<typeof createTeam>[0]) => createTeam(input),
    {
      onSuccess: (team) => {
        setSelectedTeamId(team.id);
        selectedManageTeamId.value = team.id;
        refreshTeams();
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateTeam>[1] }) =>
      updateTeam(id, input),
    { onSuccess: refreshTeams }
  );

  const deleteMutation = useApiMutation((id: string) => deleteTeam(id), {
    onSuccess: () => {
      selectedManageTeamId.value = null;
      refreshTeams();
    }
  });

  const inviteMutation = useApiMutation((teamId: string) => createTeamInvite(teamId));

  const handleNameChange = useCallback((value: string) => {
    name.value = value;
    formError.value = null;
  }, []);

  const handleNameInput = useCallback(
    (event: Event) => {
      handleNameChange((event.target as HTMLInputElement).value);
    },
    [handleNameChange]
  );

  const handleEditNameChange = useCallback((value: string) => {
    editName.value = value;
    formError.value = null;
  }, []);

  const handleEditNameInput = useCallback(
    (event: Event) => {
      handleEditNameChange((event.target as HTMLInputElement).value);
    },
    [handleEditNameChange]
  );

  const handleSelectTeam = useCallback((teamId: string) => {
    selectedManageTeamId.value = teamId;
    editingTeamId.value = null;
    inviteLink.value = null;
    inviteExpiresAt.value = null;
    formError.value = null;
  }, []);

  const handleUseTeam = useCallback((teamId: string) => {
    setSelectedTeamId(teamId);
    invalidate();
  }, []);

  const handleStartEdit = useCallback((team: Team) => {
    editingTeamId.value = team.id;
    editName.value = team.name;
    formError.value = null;
  }, []);

  const handleCancelEdit = useCallback(() => {
    editingTeamId.value = null;
    editName.value = '';
    formError.value = null;
  }, []);

  const handleCreate = useCallback(
    async (event: Event) => {
      event.preventDefault();
      formError.value = null;
      try {
        await createMutation.mutateAsync({ name: name.value });
        name.value = '';
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to create team';
      }
    },
    [createMutation]
  );

  const handleUpdate = useCallback(
    async (event: Event) => {
      event.preventDefault();
      if (!editingTeamId.value) return;
      formError.value = null;
      try {
        await updateMutation.mutateAsync({
          id: editingTeamId.value,
          input: { name: editName.value }
        });
        handleCancelEdit();
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to update team';
      }
    },
    [editingTeamId, updateMutation, handleCancelEdit]
  );

  const handleDelete = useCallback(
    async (teamId: string) => {
      formError.value = null;
      try {
        await deleteMutation.mutateAsync(teamId);
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to delete team';
      }
    },
    [deleteMutation]
  );

  const handleCreateInvite = useCallback(async () => {
    if (!selectedManageTeamId.value) return;

    formError.value = null;
    inviteLink.value = null;
    inviteExpiresAt.value = null;
    try {
      const invite = await inviteMutation.mutateAsync(selectedManageTeamId.value);
      inviteLink.value = `${window.location.origin}/invites/${invite.token}`;
      inviteExpiresAt.value = invite.expiresAt;
    } catch (err) {
      formError.value = err instanceof Error ? err.message : 'Failed to create invite';
    }
  }, [selectedManageTeamId.value, inviteMutation]);

  return {
    data: {
      teams: teamList,
      members: membersQuery.data ?? [],
      selectedTeam,
      selectedTeamId: selectedTeamId.value,
      selectedManageTeamId: selectedManageTeamId.value,
      name: name.value,
      editName: editName.value,
      editingTeamId: editingTeamId.value,
      isSingleUser: authMode?.isSingleUser ?? false,
      inviteLink: inviteLink.value,
      inviteExpiresAt: inviteExpiresAt.value
    },
    status: {
      loading,
      membersLoading: membersQuery.isLoading,
      error,
      formError: formError.value,
      creating: createMutation.isPending,
      updating: updateMutation.isPending,
      deleting: deleteMutation.isPending,
      creatingInvite: inviteMutation.isPending
    },
    actions: {
      onNameChange: handleNameChange,
      onEditNameChange: handleEditNameChange,
      onNameInput: handleNameInput,
      onEditNameInput: handleEditNameInput,
      onSelectTeam: handleSelectTeam,
      onUseTeam: handleUseTeam,
      onStartEdit: handleStartEdit,
      onCancelEdit: handleCancelEdit,
      onCreate: handleCreate,
      onUpdate: handleUpdate,
      onDelete: handleDelete,
      onCreateInvite: handleCreateInvite
    }
  };
};
