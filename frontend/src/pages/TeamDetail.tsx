import type { JSX } from 'preact';
import { PageLayout } from '../components/shared/ui/PageLayout.view';
import { TeamDetailContainer } from '../components/teams/containers/TeamDetail.container';

interface TeamDetailPageProps {
  teamId: string;
}

export const TeamDetail = ({ teamId }: TeamDetailPageProps): JSX.Element => (
  <PageLayout>
    <TeamDetailContainer teamId={teamId} />
  </PageLayout>
);
