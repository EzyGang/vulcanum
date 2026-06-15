import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  createTeam,
  deleteTeam,
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
import { formatDateTime } from '../../../utils/format';

export const useTeams = () => {
  const [, setLocation] = useLocation();
  const showCreateForm = useSignal(false);
  const name = useSignal('');
  const editName = useSignal('');
  const editingTeamId = useSignal<string | null>(null);
  const formError = useSignal<string | null>(null);

  const { data: teamList = [], isLoading: loading, error } = useApiQuery(['teams'], listTeams);

  useEffect(() => {
    storedTeams.value = teamList.map((team) => ({ id: team.id, name: team.name }));
  }, [teamList]);

  const refreshTeams = useCallback(() => {
    invalidate('teams');
  }, []);

  const createMutation = useApiMutation(
    (input: Parameters<typeof createTeam>[0]) => createTeam(input),
    {
      onSuccess: (team) => {
        setSelectedTeamId(team.id);
        showCreateForm.value = false;
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
      editingTeamId.value = null;
      refreshTeams();
    }
  });

  const handleShowCreate = useCallback(() => {
    showCreateForm.value = true;
    formError.value = null;
  }, []);

  const handleCancelCreate = useCallback(() => {
    showCreateForm.value = false;
    name.value = '';
    formError.value = null;
  }, []);

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

  const handleOpenTeam = useCallback(
    (teamId: string) => {
      setLocation(`/teams/${teamId}`);
    },
    [setLocation]
  );

  const handleOpenTeamKeyDown = useCallback(
    (event: KeyboardEvent, teamId: string) => {
      if (event.key !== 'Enter' && event.key !== ' ') return;

      event.preventDefault();
      handleOpenTeam(teamId);
    },
    [handleOpenTeam]
  );

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

  return {
    data: {
      teams: teamList.map((team) => ({
        ...team,
        formattedCreatedAt: formatDateTime(team.createdAt)
      })),
      selectedTeamId: selectedTeamId.value,
      showCreateForm,
      name: name.value,
      editName: editName.value,
      editingTeamId: editingTeamId.value
    },
    status: {
      loading,
      error,
      formError: formError.value,
      creating: createMutation.isPending,
      updating: updateMutation.isPending,
      deleting: deleteMutation.isPending
    },
    actions: {
      onShowCreate: handleShowCreate,
      onCancelCreate: handleCancelCreate,
      onNameInput: handleNameInput,
      onEditNameInput: handleEditNameInput,
      onOpenTeam: handleOpenTeam,
      onOpenTeamKeyDown: handleOpenTeamKeyDown,
      onUseTeam: handleUseTeam,
      onStartEdit: handleStartEdit,
      onCancelEdit: handleCancelEdit,
      onCreate: handleCreate,
      onUpdate: handleUpdate,
      onDelete: handleDelete
    }
  };
};
