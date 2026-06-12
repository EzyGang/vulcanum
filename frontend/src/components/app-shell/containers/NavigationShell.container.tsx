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
  const {
    navLinks,
    isActive,
    navigate,
    mobileMenuOpen,
    selectedTeamId,
    teamOptions,
    toggleMobileMenu,
    selectTeam
  } = useNavigationShell();

  return (
    <NavigationShellView
      data={{
        navLinks,
        isActive,
        mobileMenuOpen: mobileMenuOpen.value,
        selectedTeamId,
        teamOptions
      }}
      actions={{
        onLogout: logout,
        onNavigate: navigate,
        onToggleMobileMenu: toggleMobileMenu,
        onSelectTeam: selectTeam
      }}
    >
      {children}
    </NavigationShellView>
  );
};
