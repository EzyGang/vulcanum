import type { ComponentChildren, JSX } from 'preact';
import { logout } from '../../../stores/auth.store';
import { ThemeToggleContainer } from '../../theme/containers/ThemeToggle.container';

interface NavLink {
  href: string;
  label: string;
}

interface NavigationShellProps {
  children: ComponentChildren;
  data: {
    navLinks: NavLink[];
    isActive: (href: string) => boolean;
  };
}

const NavButton = ({ link, active }: { link: NavLink; active: boolean }) => (
  <a
    href={link.href}
    class={`text-sm uppercase tracking-wider transition-colors px-3 py-2 border-b-2 ${
      active
        ? 'text-text-primary border-accent'
        : 'text-text-secondary border-transparent hover:text-text-primary hover:border-border-base'
    }`}
  >
    {link.label}
  </a>
);

export const NavigationShellView = ({
  children,
  data: { navLinks, isActive }
}: NavigationShellProps): JSX.Element => (
  <div class='flex flex-col min-h-screen bg-bg-page'>
    <header class='flex items-center justify-between px-6 py-4 border-b border-border-base'>
      <div class='flex items-center gap-8'>
        <h1 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Vulcanum</h1>
        <nav class='flex items-center gap-1'>
          {navLinks.map((link) => (
            <NavButton key={link.href} link={link} active={isActive(link.href)} />
          ))}
        </nav>
      </div>
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
    <main class='flex flex-col flex-1'>{children}</main>
  </div>
);
