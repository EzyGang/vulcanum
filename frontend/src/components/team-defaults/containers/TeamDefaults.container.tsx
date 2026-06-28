import type { JSX } from 'preact';
import { useTeamDefaults } from '../hooks/useTeamDefaults.hook';
import { type TeamDefaultsSection, TeamDefaultsView } from '../ui/TeamDefaults.view';

interface TeamDefaultsContainerProps {
  teamId: string | null;
  section?: TeamDefaultsSection;
}

export const TeamDefaultsContainer = ({
  teamId,
  section = 'defaults'
}: TeamDefaultsContainerProps): JSX.Element => (
  <TeamDefaultsView {...useTeamDefaults(teamId)} section={section} />
);
