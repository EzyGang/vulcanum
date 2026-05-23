import type { JSX } from 'preact';
import { useDashboard } from '../hooks/useDashboard.hook';
import { DashboardView } from '../ui/Dashboard.view';

export const DashboardContainer = (): JSX.Element => {
  const { data, status } = useDashboard();

  return <DashboardView data={data} status={status} />;
};
