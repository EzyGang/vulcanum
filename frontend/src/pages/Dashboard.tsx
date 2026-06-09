import type { JSX } from 'preact';
import { DashboardContainer } from '../components/dashboard/containers/Dashboard.container';
import { PageLayout } from '../components/shared/ui/PageLayout.view';

export const Dashboard = (): JSX.Element => (
  <PageLayout maxWidth='6xl' gap={8}>
    <DashboardContainer />
  </PageLayout>
);
