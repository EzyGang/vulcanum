import { useSignal } from '@preact/signals';
import { useLocation } from 'wouter-preact';
import type { NavLink } from '../types';

const NAV_LINKS: NavLink[] = [
  { href: '/', label: 'Dashboard' },
  { href: '/workers', label: 'Workers' },
  { href: '/runs', label: 'Runs' },
  { href: '/settings', label: 'Settings' }
];

export const useNavigationShell = () => {
  const [location, setLocation] = useLocation();
  const mobileMenuOpen = useSignal(false);

  const isActive = (href: string): boolean => {
    if (href === '/') {
      return location === '/';
    }
    return location.startsWith(href);
  };

  const navigate = (href: string) => {
    setLocation(href);
    mobileMenuOpen.value = false;
  };

  const toggleMobileMenu = () => {
    mobileMenuOpen.value = !mobileMenuOpen.value;
  };

  return { navLinks: NAV_LINKS, isActive, navigate, mobileMenuOpen, toggleMobileMenu };
};
