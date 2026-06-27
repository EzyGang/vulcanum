import { useSignal } from '@preact/signals';
import { useEffect, useMemo } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { listProjects } from '../../../services/projects/projects.service';
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
  const { data: projectList = [], isLoading: providerProjectsLoading } = useApiQuery(
    ['task-board-projects', selectedTeamId.value],
    listTaskBoardProjects,
    { enabled: Boolean(selectedTeamId.value) }
  );
  const { data: projectConfigs = [], isLoading: projectConfigsLoading } = useApiQuery(
    ['projects', selectedTeamId.value],
    listProjects,
    { enabled: Boolean(selectedTeamId.value) }
  );
  const projectsLoading = providerProjectsLoading || projectConfigsLoading;

  const configuredProjectList = useMemo(
    () =>
      projectList.filter((project) =>
        projectConfigs.some(
          (config) =>
            config.enabled &&
            config.providerId === project.providerId &&
            config.externalProjectId === project.externalProjectId
        )
      ),
    [projectList, projectConfigs]
  );

  useEffect(() => {
    storedTeams.value = teamList.map((team) => ({ id: team.id, name: team.name }));
    const selectedStillExists = teamList.some((team) => team.id === selectedTeamId.value);
    if (!selectedStillExists && teamList[0]) {
      setSelectedTeamId(teamList[0].id);
    }
  }, [teamList, selectedTeamId.value]);

  useEffect(() => {
    if (projectsLoading) return;
    const selectedStillExists = configuredProjectList.some(
      (project) =>
        buildTaskProjectKey(project.providerId, project.externalProjectId) ===
        selectedTaskProjectKey.value
    );

    if (selectedStillExists) return;

    if (configuredProjectList[0]) {
      const firstProject = configuredProjectList[0];
      setSelectedTaskProjectKey(
        buildTaskProjectKey(firstProject.providerId, firstProject.externalProjectId)
      );
      return;
    }

    setSelectedTaskProjectKey(null);
  }, [configuredProjectList, projectsLoading, selectedTaskProjectKey.value]);

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
    projectOptions: configuredProjectList.map((project) => ({
      value: buildTaskProjectKey(project.providerId, project.externalProjectId),
      label: project.name
    })),
    toggleMobileMenu,
    selectTeam,
    selectProject
  };
};
