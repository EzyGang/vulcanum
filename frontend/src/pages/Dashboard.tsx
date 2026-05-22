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

    <main class='flex flex-col items-center justify-center flex-1 gap-12 px-6'>
      <div class='flex flex-col items-center gap-4 text-center'>
        <h2 class='text-3xl font-semibold text-text-primary'>Dashboard</h2>
        <p class='text-text-secondary text-sm max-w-md'>
          Manage workers, project configurations, and monitor work runs from the panels below.
        </p>
      </div>

      <div class='flex gap-6'>
        <a
          href='/workers'
          class='flex flex-col items-center gap-3 bg-bg-card border border-border-base p-6 hover:border-border-focus transition-colors'
        >
          <span class='text-text-primary text-sm font-medium uppercase tracking-wider'>
            Workers
          </span>
          <span class='text-text-muted text-xs'>View and manage connected workers</span>
        </a>
        <a
          href='/projects'
          class='flex flex-col items-center gap-3 bg-bg-card border border-border-base p-6 hover:border-border-focus transition-colors'
        >
          <span class='text-text-primary text-sm font-medium uppercase tracking-wider'>
            Projects
          </span>
          <span class='text-text-muted text-xs'>Configure project integrations</span>
        </a>
      </div>
    </main>
  </div>
);
