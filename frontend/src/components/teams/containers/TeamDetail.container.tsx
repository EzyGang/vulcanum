import type { JSX } from 'preact';
import { useTeamDetail } from '../hooks/useTeamDetail.hook';
import { TeamDetailView } from '../ui/TeamDetail.view';

export const TeamDetailContainer = ({ teamId }: { teamId: string }): JSX.Element => {
  const teamDetail = useTeamDetail(teamId);

  return <TeamDetailView {...teamDetail} />;
};
