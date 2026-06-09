import type { JSX } from 'preact';
import { RunsContainer } from '../components/runs/containers/Runs.container';
import { PageLayout } from '../components/shared/ui/PageLayout.view';

export const Runs = (): JSX.Element => (
  <PageLayout maxWidth='6xl'>
    <RunsContainer />
  </PageLayout>
);
