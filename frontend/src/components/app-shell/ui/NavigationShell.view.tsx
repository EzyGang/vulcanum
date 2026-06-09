import { clsx } from 'clsx';
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
    mobileMenuOpen: boolean;
  };
  actions: {
    onLogout: () => void;
    onNavigate: (href: string) => void;
    onToggleMobileMenu: () => void;
  };
}

const MobileNavButton = ({
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
    class={clsx(
      'w-full text-left text-sm uppercase tracking-wider transition-colors px-4 py-3 bg-transparent cursor-pointer border-l-2',
      active
        ? 'text-text-primary border-accent bg-bg-hover'
        : 'text-text-secondary border-transparent hover:text-text-primary hover:bg-bg-hover'
    )}
  >
    {link.label}
  </button>
);

const DesktopNavButton = ({
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
    class={clsx(
      'text-sm uppercase tracking-wider transition-colors px-3 py-2 border-b-2 bg-transparent cursor-pointer',
      active
        ? 'text-text-primary border-accent'
        : 'text-text-secondary border-transparent hover:text-text-primary hover:border-border-base'
    )}
  >
    {link.label}
  </button>
);

export const NavigationShellView = ({
  children,
  data: { navLinks, isActive, mobileMenuOpen },
  actions: { onLogout, onNavigate, onToggleMobileMenu }
}: NavigationShellProps): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-border-base relative'>
      <div class='flex items-center gap-4 sm:gap-8'>
        <div class='flex items-center gap-2 sm:gap-3'>
          <img src={LOGO_SRC} alt='Vulcanum' class='h-7 w-7 sm:h-8 sm:w-8' />
          <h1 class='text-base sm:text-lg font-semibold text-text-primary uppercase tracking-wide'>
            Vulcanum
          </h1>
        </div>
        <nav class='hidden sm:flex items-center gap-1'>
          {navLinks.map((link) => (
            <DesktopNavButton
              key={link.href}
              link={link}
              active={isActive(link.href)}
              onNavigate={onNavigate}
            />
          ))}
        </nav>
      </div>

      <div class='flex items-center gap-2 sm:gap-4'>
        <ThemeToggleContainer />
        <Button variant='ghost' onClick={onLogout}>
          Logout
        </Button>
        <button
          type='button'
          onClick={onToggleMobileMenu}
          class='sm:hidden flex items-center justify-center text-text-primary bg-transparent border-0 cursor-pointer p-2'
          aria-label='Toggle menu'
        >
          <svg
            width='18'
            height='18'
            viewBox='0 0 18 18'
            fill='none'
            stroke='currentColor'
            stroke-width='2'
            stroke-linecap='round'
          >
            <title>{mobileMenuOpen ? 'Close menu' : 'Open menu'}</title>
            {mobileMenuOpen ? (
              <path d='M4.5 4.5L13.5 13.5M13.5 4.5L4.5 13.5' />
            ) : (
              <path d='M3 5H15M3 9H15M3 13H15' />
            )}
          </svg>
        </button>
      </div>

      {mobileMenuOpen && (
        <nav class='absolute top-full left-0 right-0 bg-bg-card border-b border-border-base shadow-modal flex flex-col sm:hidden z-50 animate-slide-up'>
          {navLinks.map((link) => (
            <MobileNavButton
              key={link.href}
              link={link}
              active={isActive(link.href)}
              onNavigate={onNavigate}
            />
          ))}
        </nav>
      )}
    </header>
    <main class='flex flex-col flex-1'>{children}</main>
  </div>
);
