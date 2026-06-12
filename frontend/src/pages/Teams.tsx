import type { JSX } from 'preact';
import { PageLayout } from '../components/shared/ui/PageLayout.view';
import { TeamsContainer } from '../components/teams/containers/Teams.container';

export const Teams = (): JSX.Element => (
  <PageLayout>
    <TeamsContainer />
  </PageLayout>
);
