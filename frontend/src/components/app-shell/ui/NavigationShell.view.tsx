import type { ComponentChildren, JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { ThemeToggleContainer } from '../containers/ThemeToggle.container';
import type { NavLink } from '../types';

const LOGO_SRC = '/logo.svg';

interface NavigationShellProps {
  children: ComponentChildren;
  data: {
    navLinks: NavLink[];
    isActive: (href: string) => boolean;
  };
  actions: {
    onLogout: () => void;
    onNavigate: (href: string) => void;
  };
}

const NavButton = ({
  link,
  active,
  onNavigate
}: {
  link: NavLink;
  active: boolean;
  onNavigate: (href: string) => void;
}) => (
  <button
    type='button'
    onClick={() => onNavigate(link.href)}
    class={`text-sm uppercase tracking-wider transition-colors px-3 py-2 border-b-2 bg-transparent cursor-pointer ${
      active
        ? 'text-text-primary border-accent'
        : 'text-text-secondary border-transparent hover:text-text-primary hover:border-border-base'
    }`}
  >
    {link.label}
  </button>
);

export const NavigationShellView = ({
  children,
  data: { navLinks, isActive },
  actions: { onLogout, onNavigate }
}: NavigationShellProps): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-6 py-4 border-b border-border-base'>
      <div class='flex items-center gap-8'>
        <div class='flex items-center gap-3'>
          <img src={LOGO_SRC} alt='Vulcanum' class='h-8 w-8' />
          <h1 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Vulcanum</h1>
        </div>
        <nav class='flex items-center gap-1'>
          {navLinks.map((link) => (
            <NavButton
              key={link.href}
              link={link}
              active={isActive(link.href)}
              onNavigate={onNavigate}
            />
          ))}
        </nav>
      </div>
      <div class='flex items-center gap-4'>
        <ThemeToggleContainer />
        <Button variant='ghost' onClick={onLogout}>
          Logout
        </Button>
      </div>
    </header>
    <main class='flex flex-col flex-1'>{children}</main>
  </div>
);
