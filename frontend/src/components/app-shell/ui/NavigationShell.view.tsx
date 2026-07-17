import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { HamburgerIcon } from '../../shared/ui/HamburgerIcon.view';
import { Select } from '../../shared/ui/Select.view';
import { ThemeToggleContainer } from '../containers/ThemeToggle.container';
import type { NavLink } from '../types';

interface NavigationShellProps {
  children: ComponentChildren;
  data: {
    navLinks: NavLink[];
    isActive: (href: string) => boolean;
    mobileMenuOpen: boolean;
    selectedTeamId: string | null;
    teamOptions: { value: string; label: string }[];
    selectedProjectKey: string | null;
    boardOptions: { value: string; label: string }[];
    activatingProject: boolean;
  };
  actions: {
    onLogout: () => void;
    onNavigate: (href: string) => void;
    onSelectTeam: (teamId: string) => void;
    onSelectBoardOption: (value: string) => void;
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
  data: {
    navLinks,
    isActive,
    mobileMenuOpen,
    selectedTeamId,
    teamOptions,
    selectedProjectKey,
    boardOptions,
    activatingProject
  },
  actions: { onLogout, onNavigate, onSelectTeam, onSelectBoardOption, onToggleMobileMenu }
}: NavigationShellProps): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='sticky top-0 z-50 flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-border-base bg-bg-page'>
      <div class='flex items-center gap-4 sm:gap-8'>
        <button
          type='button'
          onClick={onToggleMobileMenu}
          class='sm:hidden flex items-center justify-center text-text-primary bg-transparent border-0 cursor-pointer p-2 -ml-2'
          aria-label='Toggle menu'
        >
          <HamburgerIcon open={mobileMenuOpen} />
        </button>
        <div class='flex items-center gap-2 sm:gap-3'>
          <svg
            viewBox='0 0 135.46666 135.46667'
            role='img'
            aria-label='Vulcanum'
            class='h-7 w-7 sm:h-8 sm:w-8'
          >
            <use href='/logo.svg#layer1' />
          </svg>
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
        <div class='hidden min-w-72 sm:block'>
          <Select
            items={boardOptions}
            value={selectedProjectKey ?? ''}
            onValueChange={onSelectBoardOption}
            placeholder={activatingProject ? 'Adding board…' : 'Select or add board'}
            disabled={activatingProject}
          />
        </div>
        {teamOptions.length > 0 && (
          <div class='hidden min-w-44 sm:block'>
            <Select
              items={teamOptions}
              value={selectedTeamId ?? ''}
              onValueChange={onSelectTeam}
              placeholder='Select team'
            />
          </div>
        )}
        <ThemeToggleContainer />
        <Button variant='ghost' onClick={onLogout}>
          Logout
        </Button>
      </div>

      {mobileMenuOpen && (
        <nav class='absolute top-full left-0 right-0 bg-bg-card border-b border-border-base shadow-modal flex flex-col sm:hidden z-50 animate-slide-up'>
          <div class='border-b border-border-base p-4'>
            <Select
              items={boardOptions}
              value={selectedProjectKey ?? ''}
              onValueChange={onSelectBoardOption}
              placeholder={activatingProject ? 'Adding board…' : 'Select or add board'}
              disabled={activatingProject}
            />
          </div>
          {teamOptions.length > 0 && (
            <div class='border-b border-border-base p-4'>
              <Select
                items={teamOptions}
                value={selectedTeamId ?? ''}
                onValueChange={onSelectTeam}
                placeholder='Select team'
              />
            </div>
          )}
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
