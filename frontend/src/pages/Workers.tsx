import type { JSX } from 'preact';
import { WorkersContainer } from '../components/workers/containers/Workers.container';
import { logout } from '../stores/auth.store';

export const Workers = (): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-6 py-4 border-b border-border-base'>
      <h1 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Vulcanum</h1>
      <div class='flex items-center gap-6'>
        <a
          href='/'
          class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors'
        >
          Dashboard
        </a>
        <button
          type='button'
          onClick={logout}
          class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors'
        >
          Logout
        </button>
      </div>
    </header>

    <main class='flex flex-col flex-1 px-6 py-8 max-w-5xl w-full mx-auto gap-6'>
      <WorkersContainer />
    </main>
  </div>
);
