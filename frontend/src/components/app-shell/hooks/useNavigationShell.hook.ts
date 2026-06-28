import { useSignal } from '@preact/signals';
import { useEffect, useMemo } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  createProject,
  listProjects as listProjectConfigs
} from '../../../services/projects/projects.service';
import {
  listProjects as listProviderProjects,
  listProviders,
  listWorkspaces
} from '../../../services/providers/providers.service';
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
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import type { NavLink } from '../types';

const NAV_LINKS: NavLink[] = [
  { href: '/', label: 'Board' },
  { href: '/teams', label: 'Teams' },
  { href: '/workers', label: 'Workers' },
  { href: '/runs', label: 'Runs' },
  { href: '/settings', label: 'Settings' }
];

const ADD_PROJECT_PREFIX = 'add:';
interface ProviderProjectCandidate {
  providerId: string;
  providerName: string;
  workspaceId: string;
  workspaceName: string;
  externalProjectId: string;
  name: string;
}

const listProviderProjectCatalog = async (): Promise<ProviderProjectCandidate[]> => {
  const providers = await listProviders();
  const providerProjects = await Promise.all(
    providers.map(async (provider) => {
      const workspaces = await listWorkspaces(provider.id);
      const workspaceProjects = await Promise.all(
        workspaces.map(async (workspace) => {
          const projects = await listProviderProjects(provider.id, workspace.id);
          return projects.map((project) => ({
            providerId: provider.id,
            providerName: provider.name,
            workspaceId: workspace.id,
            workspaceName: workspace.name,
            externalProjectId: project.id,
            name: project.name
          }));
        })
      );

      return workspaceProjects.flat();
    })
  );

  return providerProjects.flat();
};

export const useNavigationShell = () => {
  const [location, setLocation] = useLocation();
  const mobileMenuOpen = useSignal(false);
  const { data: teamList = [] } = useApiQuery(['teams'], listTeams);
  const { data: projectConfigs = [], isLoading: projectConfigsLoading } = useApiQuery(
    ['projects', selectedTeamId.value],
    listProjectConfigs,
    { enabled: Boolean(selectedTeamId.value) }
  );
  const { data: providerProjectCatalog = [] } = useApiQuery(
    ['provider-project-catalog', selectedTeamId.value],
    listProviderProjectCatalog,
    { enabled: Boolean(selectedTeamId.value) }
  );
  const projectsLoading = projectConfigsLoading;

  const configuredProjectList = useMemo(
    () =>
      projectConfigs
        .filter((config) => Boolean(config.providerId))
        .map((config) => ({
          providerId: config.providerId ?? '',
          externalProjectId: config.externalProjectId,
          name: config.name || config.externalProjectId
        })),
    [projectConfigs]
  );

  const availableProjectList = useMemo(
    () =>
      providerProjectCatalog.filter(
        (project) =>
          !projectConfigs.some(
            (config) =>
              config.providerId === project.providerId &&
              config.externalProjectId === project.externalProjectId
          )
      ),
    [providerProjectCatalog, projectConfigs]
  );
  const boardOptions = useMemo(
    () => [
      ...configuredProjectList.map((project) => ({
        value: buildTaskProjectKey(project.providerId, project.externalProjectId),
        label: project.name
      })),
      ...availableProjectList.map((project) => ({
        value: `${ADD_PROJECT_PREFIX}${buildTaskProjectKey(
          project.providerId,
          project.externalProjectId
        )}`,
        label: `Add ${project.name} · ${project.workspaceName} · ${project.providerName}`
      }))
    ],
    [configuredProjectList, availableProjectList]
  );

  const activateProjectMutation = useApiMutation(
    (projectKey: string) => {
      const project = availableProjectList.find(
        (candidate) =>
          buildTaskProjectKey(candidate.providerId, candidate.externalProjectId) === projectKey
      );
      if (!project) {
        throw new Error('Provider project is no longer available');
      }

      return createProject({
        providerId: project.providerId,
        externalProjectId: project.externalProjectId,
        externalWorkspaceId: project.workspaceId,
        name: project.name,
        enabled: false
      });
    },
    {
      onSuccess: async (config) => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: ['projects'] }),
          queryClient.invalidateQueries({ queryKey: ['task-board-projects'] }),
          queryClient.invalidateQueries({ queryKey: ['provider-project-catalog'] })
        ]);
        if (config.providerId) {
          setSelectedTaskProjectKey(
            buildTaskProjectKey(config.providerId, config.externalProjectId)
          );
          setLocation('/');
          mobileMenuOpen.value = false;
        }
      }
    }
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

  const activateProject = (projectKey: string) => {
    if (!projectKey) return;
    activateProjectMutation.mutate(projectKey);
  };

  const selectBoardOption = (value: string) => {
    if (!value) return;
    if (value.startsWith(ADD_PROJECT_PREFIX)) {
      activateProject(value.slice(ADD_PROJECT_PREFIX.length));
      return;
    }

    selectProject(value);
  };

  return {
    navLinks: NAV_LINKS,
    isActive,
    navigate,
    mobileMenuOpen,
    selectedTeamId: selectedTeamId.value,
    teamOptions: teamList.map((team) => ({ value: team.id, label: team.name })),
    selectedProjectKey: selectedTaskProjectKey.value,
    boardOptions,
    activatingProject: activateProjectMutation.isPending,
    toggleMobileMenu,
    selectTeam,
    selectBoardOption,
    activateProject
  };
};
