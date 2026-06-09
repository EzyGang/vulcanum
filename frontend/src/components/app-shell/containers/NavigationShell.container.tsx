import type { ComponentChildren, JSX } from 'preact';
import { logout } from '../../../stores/auth.store';
import { useNavigationShell } from '../hooks/useNavigationShell.hook';
import { NavigationShellView } from '../ui/NavigationShell.view';

interface NavigationShellContainerProps {
  children: ComponentChildren;
}

export const NavigationShellContainer = ({
  children
}: NavigationShellContainerProps): JSX.Element => {
  const { navLinks, isActive, navigate, mobileMenuOpen, toggleMobileMenu } = useNavigationShell();

  return (
    <NavigationShellView
      data={{ navLinks, isActive, mobileMenuOpen: mobileMenuOpen.value }}
      actions={{ onLogout: logout, onNavigate: navigate, onToggleMobileMenu: toggleMobileMenu }}
    >
      {children}
    </NavigationShellView>
  );
};
