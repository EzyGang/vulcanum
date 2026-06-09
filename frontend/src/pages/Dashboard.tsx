import type { JSX } from 'preact';
import { DashboardContainer } from '../components/dashboard/containers/Dashboard.container';

export const Dashboard = (): JSX.Element => (
  <div class='flex flex-col flex-1 px-4 sm:px-6 py-8 max-w-6xl w-full mx-auto gap-8 animate-fade-in'>
    <DashboardContainer />
  </div>
);
