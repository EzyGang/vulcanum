import type { JSX } from 'preact';
import type { Team, TeamMember } from '../../../types/teams';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Table } from '../../shared/ui/Table.view';

interface TeamsViewProps {
  data: {
    teams: Team[];
    members: TeamMember[];
    selectedTeam: Team | null;
    selectedTeamId: string | null;
    selectedManageTeamId: string | null;
    name: string;
    editName: string;
    editingTeamId: string | null;
    isSingleUser: boolean;
  };
  status: {
    loading: boolean;
    membersLoading: boolean;
    error: ApiError | null;
    formError: string | null;
    creating: boolean;
    updating: boolean;
    deleting: boolean;
  };
  actions: {
    onNameChange: (value: string) => void;
    onEditNameChange: (value: string) => void;
    onSelectTeam: (teamId: string) => void;
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
    <div class='flex flex-col gap-2'>
      <h3 class='text-base font-semibold text-text-secondary uppercase tracking-wide'>Teams</h3>
      <p class='text-sm text-text-muted'>Teams scope providers, projects, workers, and runs.</p>
    </div>

    {status.error && <ErrorBanner message={status.error.message} />}
    {status.formError && <ErrorBanner message={status.formError} />}

    <form
      onSubmit={actions.onCreate}
      class='flex flex-col gap-3 border border-border-base bg-bg-card p-5 sm:flex-row'
    >
      <Input
        value={data.name}
        onInput={(event) => actions.onNameChange((event.target as HTMLInputElement).value)}
        placeholder='New team name'
        disabled={status.creating}
      />
      <Button type='submit' variant='primary' disabled={status.creating}>
        {status.creating ? 'Creating...' : 'Create Team'}
      </Button>
    </form>

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
              class={team.id === data.selectedManageTeamId ? 'bg-bg-active' : ''}
            >
              <Table.Cell>
                <button
                  type='button'
                  class='bg-transparent border-0 p-0 text-left text-text-primary text-sm cursor-pointer hover:underline'
                  onClick={() => actions.onSelectTeam(team.id)}
                >
                  {team.name}
                </button>
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
                <span class='text-sm text-text-secondary'>{team.createdAt}</span>
              </Table.Cell>
              <Table.Cell>
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

    {data.selectedTeam && (
      <div class='flex flex-col gap-5 border border-border-base bg-bg-card p-5'>
        <div class='flex flex-col gap-1'>
          <h4 class='text-sm font-semibold uppercase tracking-wide text-text-primary'>
            Selected Team
          </h4>
          <span class='font-mono text-xs text-text-muted'>{data.selectedTeam.id}</span>
        </div>

        {data.editingTeamId === data.selectedTeam.id && (
          <form onSubmit={actions.onUpdate} class='flex flex-col gap-3 sm:flex-row'>
            <Input
              value={data.editName}
              onInput={(event) =>
                actions.onEditNameChange((event.target as HTMLInputElement).value)
              }
              disabled={status.updating}
            />
            <Button type='submit' variant='primary' disabled={status.updating}>
              {status.updating ? 'Saving...' : 'Save'}
            </Button>
            <Button type='button' variant='secondary' onClick={actions.onCancelEdit}>
              Cancel
            </Button>
          </form>
        )}

        <div class='flex flex-col gap-3'>
          <h4 class='text-sm font-semibold uppercase tracking-wide text-text-primary'>Members</h4>
          {data.isSingleUser && (
            <div class='border border-border-base bg-bg-panel p-4 text-sm text-text-muted'>
              Member management requires multiuser authentication. Instance-password deployments can
              still use teams as scopes.
            </div>
          )}
          {status.membersLoading && <div class='text-sm text-text-muted'>Loading members...</div>}
          {!status.membersLoading && data.members.length === 0 && (
            <div class='text-sm text-text-muted'>No members are attached to this team.</div>
          )}
          {!status.membersLoading && data.members.length > 0 && (
            <Table>
              <Table.Head>
                <Table.HeadCell>Email</Table.HeadCell>
                <Table.HeadCell>User ID</Table.HeadCell>
                <Table.HeadCell>Role</Table.HeadCell>
                <Table.HeadCell>Created</Table.HeadCell>
              </Table.Head>
              <Table.Body>
                {data.members.map((member) => (
                  <Table.Row key={`${member.teamId}:${member.userId}`}>
                    <Table.Cell>
                      <span class='text-sm text-text-primary'>{member.email}</span>
                    </Table.Cell>
                    <Table.Cell>
                      <span class='font-mono text-xs text-text-muted'>{member.userId}</span>
                    </Table.Cell>
                    <Table.Cell>
                      <span class='text-sm text-text-secondary'>{member.role}</span>
                    </Table.Cell>
                    <Table.Cell>
                      <span class='text-sm text-text-secondary'>{member.createdAt}</span>
                    </Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table>
          )}
        </div>

        <div class='flex flex-col gap-3'>
          <h4 class='text-sm font-semibold uppercase tracking-wide text-text-primary'>Invites</h4>
          <div class='border border-border-base bg-bg-panel p-4 text-sm text-text-muted'>
            Invites are not implemented yet. Future options are GitHub identity invites or generated
            invite links.
          </div>
          <Button type='button' variant='secondary' disabled>
            Create Invite
          </Button>
        </div>
      </div>
    )}
  </div>
);
