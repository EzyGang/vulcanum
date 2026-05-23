import type { JSX } from 'preact';
import { ThemeToggleContainer } from '../components/theme/containers/ThemeToggle.container';
import { logout } from '../stores/auth.store';

export const Dashboard = (): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-6 py-4 border-b border-border-base'>
      <h1 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Vulcanum</h1>
      <div class='flex items-center gap-4'>
        <ThemeToggleContainer />
        <button
          type='button'
          onClick={logout}
          class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors'
        >
          Logout
        </button>
      </div>
    </header>

    <main class='flex flex-col items-center justify-center flex-1 gap-6 px-6'>
      <div class='flex flex-col items-center gap-4 text-center'>
        <h2 class='text-3xl font-semibold text-text-primary'>Dashboard</h2>
        <p class='text-text-muted text-sm'>More features coming soon.</p>
      </div>
    </main>
  </div>
);
