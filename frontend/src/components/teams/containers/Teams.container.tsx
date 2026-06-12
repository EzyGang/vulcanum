import type { JSX } from 'preact';
import { useTeams } from '../hooks/useTeams.hook';
import { TeamsView } from '../ui/Teams.view';

export const TeamsContainer = (): JSX.Element => {
  const teams = useTeams();

  return <TeamsView {...teams} />;
};
