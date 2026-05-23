import type { ComponentChildren, JSX } from 'preact';
import { logout } from '../../../stores/auth.store';
import { ThemeToggleContainer } from '../../theme/containers/ThemeToggle.container';

interface NavLink {
  href: string;
  label: string;
}

interface PageLayoutProps {
  children: ComponentChildren;
  navLinks?: NavLink[];
}

export const PageLayout = ({ children, navLinks = [] }: PageLayoutProps): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-6 py-4 border-b border-border-base'>
      <h1 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Vulcanum</h1>
      <div class='flex items-center gap-6'>
        {navLinks.map((link) => (
          <a
            key={link.href}
            href={link.href}
            class='text-text-secondary text-sm uppercase tracking-wider hover:text-text-primary transition-colors'
          >
            {link.label}
          </a>
        ))}
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

    <main class='flex flex-col flex-1'>{children}</main>
  </div>
);
