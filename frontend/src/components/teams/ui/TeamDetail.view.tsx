import type { JSX } from 'preact';
import type { Team, TeamMember } from '../../../types/teams';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
import { Table } from '../../shared/ui/Table.view';

interface FormattedTeam extends Team {
  formattedCreatedAt: string;
}

interface FormattedMember extends TeamMember {
  formattedCreatedAt: string;
}

interface TeamDetailViewProps {
  data: {
    team: FormattedTeam | null;
    members: FormattedMember[];
    selectedTeamId: string | null;
    editName: string;
    editing: boolean;
    isSingleUser: boolean;
    inviteLink: string | null;
    inviteExpiresAt: string | null;
  };
  status: {
    loading: boolean;
    membersLoading: boolean;
    error: ApiError | null;
    formError: string | null;
    updating: boolean;
    deleting: boolean;
    creatingInvite: boolean;
  };
  actions: {
    onBack: () => void;
    onUseTeam: () => void;
    onStartEdit: () => void;
    onCancelEdit: () => void;
    onEditNameInput: (event: Event) => void;
    onUpdate: (event: Event) => void;
    onDelete: () => void;
    onCreateInvite: () => void;
  };
}

export const TeamDetailView = ({ data, status, actions }: TeamDetailViewProps): JSX.Element => (
  <div class='flex flex-col gap-6'>
    <SectionHeader
      title={data.team?.name ?? 'Team'}
      hint='Manage team details, members, and invites.'
      action={
        <Button variant='secondary' onClick={actions.onBack}>
          Back to Teams
        </Button>
      }
    />

    {status.error && <ErrorBanner message={status.error.message} />}
    {status.formError && <ErrorBanner message={status.formError} />}
    {status.loading && <div class='text-sm text-text-muted'>Loading team...</div>}

    {!status.loading && data.team && (
      <>
        <div class='flex flex-col gap-4 border border-border-base bg-bg-card p-5'>
          <div class='grid grid-cols-1 gap-4 md:grid-cols-2'>
            <div class='flex flex-col gap-1'>
              <span class='text-xs uppercase tracking-wider text-text-muted'>Team ID</span>
              <span class='font-mono text-xs text-text-secondary'>{data.team.id}</span>
            </div>
            <div class='flex flex-col gap-1'>
              <span class='text-xs uppercase tracking-wider text-text-muted'>Created</span>
              <span class='text-sm text-text-secondary'>{data.team.formattedCreatedAt}</span>
            </div>
            <div class='flex flex-col gap-1'>
              <span class='text-xs uppercase tracking-wider text-text-muted'>Personal User ID</span>
              <span class='font-mono text-xs text-text-secondary'>
                {data.team.personalUserId ?? 'none'}
              </span>
            </div>
            <div class='flex flex-col gap-1'>
              <span class='text-xs uppercase tracking-wider text-text-muted'>Current Scope</span>
              <span class='text-sm text-text-secondary'>
                {data.team.id === data.selectedTeamId ? 'Current team' : 'Not selected'}
              </span>
            </div>
          </div>

          {data.editing ? (
            <form onSubmit={actions.onUpdate} class='flex flex-col gap-3 sm:flex-row'>
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
            <div class='flex flex-wrap items-center gap-3'>
              <Button
                variant='secondary'
                onClick={actions.onUseTeam}
                disabled={data.team.id === data.selectedTeamId}
              >
                {data.team.id === data.selectedTeamId ? 'Current Team' : 'Use Team'}
              </Button>
              <Button variant='secondary' onClick={actions.onStartEdit}>
                Rename
              </Button>
              <Button variant='ghost-danger' onClick={actions.onDelete} disabled={status.deleting}>
                {status.deleting ? 'Deleting...' : 'Delete Team'}
              </Button>
            </div>
          )}
        </div>

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
                      <span class='text-sm text-text-secondary'>{member.formattedCreatedAt}</span>
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
            Generate a single-use invite link for this team. Links expire after 30 minutes and can
            be used by GitHub-authenticated users only.
          </div>
          {data.inviteLink && (
            <div class='flex flex-col gap-2 border border-border-base bg-bg-panel p-4'>
              <span class='text-xs uppercase tracking-wide text-text-muted'>Generated Link</span>
              <span class='break-all font-mono text-xs text-text-primary'>{data.inviteLink}</span>
              {data.inviteExpiresAt && (
                <span class='text-xs text-text-muted'>Expires at {data.inviteExpiresAt}</span>
              )}
            </div>
          )}
          <Button
            type='button'
            variant='secondary'
            onClick={actions.onCreateInvite}
            disabled={data.isSingleUser || status.creatingInvite}
          >
            {status.creatingInvite ? 'Creating Invite...' : 'Create Invite'}
          </Button>
        </div>
      </>
    )}
  </div>
);
