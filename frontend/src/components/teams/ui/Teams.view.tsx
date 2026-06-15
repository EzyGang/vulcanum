import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { Team } from '../../../types/teams';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
import { Table } from '../../shared/ui/Table.view';

interface TeamRow extends Team {
  formattedCreatedAt: string;
}

interface TeamsViewProps {
  data: {
    teams: TeamRow[];
    selectedTeamId: string | null;
    showCreateForm: Signal<boolean>;
    name: string;
    editName: string;
    editingTeamId: string | null;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
    formError: string | null;
    creating: boolean;
    updating: boolean;
    deleting: boolean;
  };
  actions: {
    onShowCreate: () => void;
    onCancelCreate: () => void;
    onNameInput: (event: Event) => void;
    onEditNameInput: (event: Event) => void;
    onOpenTeam: (teamId: string) => void;
    onOpenTeamKeyDown: (event: KeyboardEvent, teamId: string) => void;
    onUseTeam: (teamId: string) => void;
    onStartEdit: (team: Team) => void;
    onCancelEdit: () => void;
    onCreate: (event: Event) => void;
    onUpdate: (event: Event) => void;
    onDelete: (teamId: string) => void;
  };
}

export const TeamsView = ({ data, status, actions }: TeamsViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <SectionHeader
      title='Teams'
      hint='Teams scope providers, projects, workers, and runs.'
      action={
        !data.showCreateForm.value ? (
          <Button
            variant='primary'
            class='shrink-0 whitespace-nowrap px-5'
            onClick={actions.onShowCreate}
          >
            Create Team
          </Button>
        ) : null
      }
    />

    {status.error && <ErrorBanner message={status.error.message} />}
    {status.formError && <ErrorBanner message={status.formError} />}

    {data.showCreateForm.value && (
      <form
        onSubmit={actions.onCreate}
        class='flex flex-col gap-3 border border-border-base bg-bg-card p-5 sm:flex-row'
      >
        <Input
          value={data.name}
          onInput={actions.onNameInput}
          placeholder='New team name'
          disabled={status.creating}
        />
        <Button type='submit' variant='primary' disabled={status.creating}>
          {status.creating ? 'Creating...' : 'Create Team'}
        </Button>
        <Button type='button' variant='secondary' onClick={actions.onCancelCreate}>
          Cancel
        </Button>
      </form>
    )}

    {status.loading && <div class='text-text-muted text-sm'>Loading teams...</div>}

    {!status.loading && data.teams.length === 0 && (
      <EmptyState title='No teams found.' description='Create a team to scope work.' />
    )}

    {!status.loading && data.teams.length > 0 && (
      <Table>
        <Table.Head>
          <Table.HeadCell>Name</Table.HeadCell>
          <Table.HeadCell>ID</Table.HeadCell>
          <Table.HeadCell>Personal User ID</Table.HeadCell>
          <Table.HeadCell>Created</Table.HeadCell>
          <Table.HeadCell>Actions</Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {data.teams.map((team) => (
            <Table.Row
              key={team.id}
              class={
                team.id === data.selectedTeamId ? 'cursor-pointer bg-bg-active' : 'cursor-pointer'
              }
              role='button'
              tabIndex={0}
              onClick={() => actions.onOpenTeam(team.id)}
              onKeyDown={(event) => actions.onOpenTeamKeyDown(event, team.id)}
            >
              <Table.Cell onClick={(event) => event.stopPropagation()}>
                {data.editingTeamId === team.id ? (
                  <form onSubmit={actions.onUpdate} class='flex flex-col gap-2 sm:flex-row'>
                    <Input
                      value={data.editName}
                      onInput={actions.onEditNameInput}
                      disabled={status.updating}
                    />
                    <Button type='submit' variant='primary' disabled={status.updating}>
                      {status.updating ? 'Saving...' : 'Save'}
                    </Button>
                    <Button type='button' variant='secondary' onClick={actions.onCancelEdit}>
                      Cancel
                    </Button>
                  </form>
                ) : (
                  <span class='text-text-primary text-sm'>{team.name}</span>
                )}
              </Table.Cell>
              <Table.Cell>
                <span class='font-mono text-xs text-text-muted'>{team.id}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='font-mono text-xs text-text-muted'>
                  {team.personalUserId ?? 'none'}
                </span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-sm text-text-secondary'>{team.formattedCreatedAt}</span>
              </Table.Cell>
              <Table.Cell onClick={(event) => event.stopPropagation()}>
                <div class='flex flex-wrap items-center gap-3'>
                  <Button
                    variant='ghost'
                    onClick={() => actions.onUseTeam(team.id)}
                    disabled={team.id === data.selectedTeamId}
                  >
                    {team.id === data.selectedTeamId ? 'Current' : 'Use'}
                  </Button>
                  <Button variant='ghost' onClick={() => actions.onStartEdit(team)}>
                    Rename
                  </Button>
                  <Button
                    variant='ghost-danger'
                    onClick={() => actions.onDelete(team.id)}
                    disabled={status.deleting}
                  >
                    Delete
                  </Button>
                </div>
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table>
    )}
  </div>
);
