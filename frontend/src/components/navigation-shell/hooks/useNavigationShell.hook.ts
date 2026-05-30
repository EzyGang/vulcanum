import { useLocation } from 'wouter-preact';
import type { NavLink } from '../types';

const NAV_LINKS: NavLink[] = [
  { href: '/', label: 'Dashboard' },
  { href: '/workers', label: 'Workers' },
  { href: '/providers', label: 'Providers' },
  { href: '/projects', label: 'Projects' },
  { href: '/runs', label: 'Runs' }
];

export const useNavigationShell = () => {
  const [location] = useLocation();

  const isActive = (href: string): boolean => {
    if (href === '/') {
      return location === '/';
    }
    return location.startsWith(href);
  };

  return { navLinks: NAV_LINKS, isActive };
};
