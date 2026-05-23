import type { JSX } from 'preact';
import { PageLayout } from '../components/layout/ui/PageLayout.view';

export const Dashboard = (): JSX.Element => (
  <PageLayout navLinks={[{ href: '/workers', label: 'Workers' }]}>
    <div class='flex flex-col items-center justify-center flex-1 gap-6 px-6'>
      <div class='flex flex-col items-center gap-4 text-center'>
        <h2 class='text-3xl font-semibold text-text-primary'>Dashboard</h2>
        <p class='text-text-muted text-sm'>More features coming soon.</p>
      </div>
    </div>
  </PageLayout>
);
