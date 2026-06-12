import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { listTeams } from '../../../services/teams/teams.service';
import {
  selectedTeamId,
  setSelectedTeamId,
  teams as storedTeams
} from '../../../stores/auth.store';
import { queryClient } from '../../../utils/api/query/client';
import { useApiQuery } from '../../../utils/api/query/hooks';
import type { NavLink } from '../types';

const NAV_LINKS: NavLink[] = [
  { href: '/', label: 'Dashboard' },
  { href: '/teams', label: 'Teams' },
  { href: '/workers', label: 'Workers' },
  { href: '/runs', label: 'Runs' },
  { href: '/settings', label: 'Settings' }
];

export const useNavigationShell = () => {
  const [location, setLocation] = useLocation();
  const mobileMenuOpen = useSignal(false);
  const { data: teamList = [] } = useApiQuery(['teams'], listTeams);

  useEffect(() => {
    storedTeams.value = teamList.map((team) => ({ id: team.id, name: team.name }));
    const selectedStillExists = teamList.some((team) => team.id === selectedTeamId.value);
    if (!selectedStillExists && teamList[0]) {
      setSelectedTeamId(teamList[0].id);
    }
  }, [teamList, selectedTeamId.value]);

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

  const selectTeam = (teamId: string) => {
    setSelectedTeamId(teamId);
    queryClient.invalidateQueries({ refetchType: 'active' });
  };

  return {
    navLinks: NAV_LINKS,
    isActive,
    navigate,
    mobileMenuOpen,
    selectedTeamId: selectedTeamId.value,
    teamOptions: teamList.map((team) => ({ value: team.id, label: team.name })),
    toggleMobileMenu,
    selectTeam
  };
};
