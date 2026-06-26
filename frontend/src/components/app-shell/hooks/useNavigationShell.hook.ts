import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { listTaskBoardProjects } from '../../../services/task-board/task-board.service';
import { listTeams } from '../../../services/teams/teams.service';
import {
  selectedTeamId,
  setSelectedTeamId,
  teams as storedTeams
} from '../../../stores/auth.store';
import {
  buildTaskProjectKey,
  selectedTaskProjectKey,
  setSelectedTaskProjectKey
} from '../../../stores/task-board.store';
import { queryClient } from '../../../utils/api/query/client';
import { useApiQuery } from '../../../utils/api/query/hooks';
import type { NavLink } from '../types';

const NAV_LINKS: NavLink[] = [
  { href: '/', label: 'Board' },
  { href: '/teams', label: 'Teams' },
  { href: '/workers', label: 'Workers' },
  { href: '/runs', label: 'Runs' },
  { href: '/settings', label: 'Settings' }
];

export const useNavigationShell = () => {
  const [location, setLocation] = useLocation();
  const mobileMenuOpen = useSignal(false);
  const { data: teamList = [] } = useApiQuery(['teams'], listTeams);
  const { data: projectList = [] } = useApiQuery(
    ['task-board-projects', selectedTeamId.value],
    listTaskBoardProjects,
    { enabled: Boolean(selectedTeamId.value) }
  );

  useEffect(() => {
    storedTeams.value = teamList.map((team) => ({ id: team.id, name: team.name }));
    const selectedStillExists = teamList.some((team) => team.id === selectedTeamId.value);
    if (!selectedStillExists && teamList[0]) {
      setSelectedTeamId(teamList[0].id);
    }
  }, [teamList, selectedTeamId.value]);

  useEffect(() => {
    const selectedStillExists = projectList.some(
      (project) =>
        buildTaskProjectKey(project.providerId, project.externalProjectId) ===
        selectedTaskProjectKey.value
    );

    if (selectedStillExists) return;

    if (projectList[0]) {
      setSelectedTaskProjectKey(
        buildTaskProjectKey(projectList[0].providerId, projectList[0].externalProjectId)
      );
      return;
    }

    setSelectedTaskProjectKey(null);
  }, [projectList, selectedTaskProjectKey.value]);

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

  const selectProject = (projectKey: string) => {
    setSelectedTaskProjectKey(projectKey);
    setLocation('/');
    mobileMenuOpen.value = false;
  };

  return {
    navLinks: NAV_LINKS,
    isActive,
    navigate,
    mobileMenuOpen,
    selectedTeamId: selectedTeamId.value,
    teamOptions: teamList.map((team) => ({ value: team.id, label: team.name })),
    selectedProjectKey: selectedTaskProjectKey.value,
    projectOptions: projectList.map((project) => ({
      value: buildTaskProjectKey(project.providerId, project.externalProjectId),
      label: project.name
    })),
    toggleMobileMenu,
    selectTeam,
    selectProject
  };
};
