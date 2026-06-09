import type { JSX } from 'preact';
import { PageLayout } from '../components/shared/ui/PageLayout.view';
import { WorkersContainer } from '../components/workers/containers/Workers.container';

export const Workers = (): JSX.Element => (
  <PageLayout>
    <WorkersContainer />
  </PageLayout>
);
