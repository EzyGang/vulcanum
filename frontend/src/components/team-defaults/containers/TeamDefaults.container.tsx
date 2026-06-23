import type { JSX } from 'preact';
import { useTeamDefaults } from '../hooks/useTeamDefaults.hook';
import { TeamDefaultsView } from '../ui/TeamDefaults.view';

export const TeamDefaultsContainer = ({ teamId }: { teamId: string | null }): JSX.Element => {
  const { data, status, actions } = useTeamDefaults(teamId);

  return <TeamDefaultsView data={data} status={status} actions={actions} />;
};
