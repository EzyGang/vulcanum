import type { JSX } from 'preact';
import { TeamDefaultsContainer } from '../../team-defaults/containers/TeamDefaults.container';
import { useTeamDetail } from '../hooks/useTeamDetail.hook';
import { TeamDetailView } from '../ui/TeamDetail.view';

export const TeamDetailContainer = ({ teamId }: { teamId: string }): JSX.Element => {
  const teamDetail = useTeamDetail(teamId);
  const teamDefaults = teamDetail.data.team ? (
    <TeamDefaultsContainer teamId={teamDetail.data.team.id} />
  ) : null;

  return <TeamDetailView content={{ teamDefaults }} {...teamDetail} />;
};
