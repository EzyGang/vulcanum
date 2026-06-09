import type { Signal } from '@preact/signals';
import { computed, useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import {
  listProjects,
  listWorkspaces,
  lookupProject
} from '../../../../services/providers/providers.service';
import type { ColumnInfo, ProjectInfo, WorkspaceInfo } from '../../../../types/projects';

export const useProjectFormLookup = (
  providerId: Signal<string>,
  externalProjectId: Signal<string>,
  isEdit: boolean,
  isSubmitting: Signal<boolean>
) => {
  const columns = useSignal<ColumnInfo[]>([]);
  const columnsLoading = useSignal(false);
  const lookupProjectName = useSignal('');
  const lookupError = useSignal<string | null>(null);
  const lookedUp = useSignal(false);
  const workspaces = useSignal<WorkspaceInfo[]>([]);
  const workspacesLoading = useSignal(false);
  const workspaceId = useSignal('');
  const projects = useSignal<ProjectInfo[]>([]);
  const projectsLoading = useSignal(false);

  const workspaceOptions = computed(() =>
    workspaces.value.map((w) => ({ value: w.id, label: w.name }))
  );

  const projectOptions = computed(() =>
    projects.value.map((pr) => ({ value: pr.id, label: `${pr.name} (${pr.slug})` }))
  );

  const workspaceSelectDisabled = computed(
    () => isEdit || isSubmitting.value || workspacesLoading.value || !providerId.value
  );

  const projectSelectDisabled = computed(
    () => isEdit || isSubmitting.value || projectsLoading.value
  );

  const handleLookup = useCallback(async () => {
    if (!providerId.value || !externalProjectId.value) return;

    lookupError.value = null;
    columnsLoading.value = true;
    lookedUp.value = false;

    try {
      const result = await lookupProject(providerId.value, externalProjectId.value);
      lookupProjectName.value = result.name;
      columns.value = result.columns;
      lookedUp.value = true;
    } catch (err) {
      lookupError.value = err instanceof Error ? err.message : 'Lookup failed';
      columns.value = [];
      lookupProjectName.value = '';
    } finally {
      columnsLoading.value = false;
    }
  }, []);

  const handleWorkspaceChange = useCallback(async (id: string) => {
    workspaceId.value = id;
    projects.value = [];
    resetLookupInternals();
    externalProjectId.value = '';

    if (!id || !providerId.value) return;

    projectsLoading.value = true;
    try {
      projects.value = await listProjects(providerId.value, id);
    } catch {
      projects.value = [];
    } finally {
      projectsLoading.value = false;
    }
  }, []);

  const handleProjectSelect = useCallback(
    (project: ProjectInfo) => {
      externalProjectId.value = project.id;
      lookupProjectName.value = project.name;
      handleLookup();
    },
    [handleLookup]
  );

  const handleProjectSelectById = useCallback(
    (id: string) => {
      const project = projects.value.find((pr) => pr.id === id);
      if (project) {
        handleProjectSelect(project);
      }
    },
    [handleProjectSelect, projects]
  );

  const fetchWorkspaces = useCallback(async () => {
    if (!providerId.value) {
      workspaces.value = [];
      return;
    }

    workspacesLoading.value = true;
    try {
      workspaces.value = await listWorkspaces(providerId.value);
    } catch {
      workspaces.value = [];
    } finally {
      workspacesLoading.value = false;
    }
  }, []);

  const resetLookup = useCallback(() => {
    resetLookupInternals();
    workspaces.value = [];
    workspaceId.value = '';
    projects.value = [];
  }, []);

  const resetLookupInternals = () => {
    lookedUp.value = false;
    columns.value = [];
    lookupProjectName.value = '';
    lookupError.value = null;
  };

  return {
    columns,
    columnsLoading,
    lookupProjectName,
    lookupError,
    lookedUp,
    handleLookup,
    resetLookup,
    workspaceOptions,
    workspacesLoading,
    workspaceId,
    projectOptions,
    projects,
    projectsLoading,
    workspaceSelectDisabled,
    projectSelectDisabled,
    handleWorkspaceChange,
    handleProjectSelect,
    handleProjectSelectById,
    fetchWorkspaces
  };
};
