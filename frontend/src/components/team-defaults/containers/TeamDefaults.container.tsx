import type { JSX } from 'preact';
import { useTeamDefaults } from '../hooks/useTeamDefaults.hook';
import { TeamDefaultsView } from '../ui/TeamDefaults.view';

export const TeamDefaultsContainer = ({ teamId }: { teamId: string | null }): JSX.Element => (
  <TeamDefaultsView {...useTeamDefaults(teamId)} />
);
