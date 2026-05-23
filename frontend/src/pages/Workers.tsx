import type { JSX } from 'preact';
import { PageLayout } from '../components/layout/ui/PageLayout.view';
import { WorkersContainer } from '../components/workers/containers/Workers.container';

export const Workers = (): JSX.Element => (
  <PageLayout navLinks={[{ href: '/', label: 'Dashboard' }]}>
    <div class='flex flex-col flex-1 px-6 py-8 max-w-5xl w-full mx-auto gap-6'>
      <WorkersContainer />
    </div>
  </PageLayout>
);
