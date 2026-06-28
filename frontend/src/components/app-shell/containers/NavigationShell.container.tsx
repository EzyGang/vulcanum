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
    selectedProjectKey,
    boardOptions,
    activatingProject,
    toggleMobileMenu,
    selectTeam,
    selectBoardOption
  } = useNavigationShell();

  return (
    <NavigationShellView
      data={{
        navLinks,
        isActive,
        mobileMenuOpen: mobileMenuOpen.value,
        selectedTeamId,
        teamOptions,
        selectedProjectKey,
        boardOptions,
        activatingProject
      }}
      actions={{
        onLogout: logout,
        onNavigate: navigate,
        onToggleMobileMenu: toggleMobileMenu,
        onSelectTeam: selectTeam,
        onSelectBoardOption: selectBoardOption
      }}
    >
      {children}
    </NavigationShellView>
  );
};
