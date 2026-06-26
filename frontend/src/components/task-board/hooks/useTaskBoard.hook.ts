import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { listRepos } from '../../../services/github/github.service';
import {
  createProject,
  listProjects,
  updateProject
} from '../../../services/projects/projects.service';
import {
  createTask,
  getTaskBoard,
  listTaskBoardProjects,
  moveTask
} from '../../../services/task-board/task-board.service';
import { parseTaskProjectKey, selectedTaskProjectKey } from '../../../stores/task-board.store';
import type { ProjectConfig } from '../../../types/projects';
import type { TaskBoardColumn, TaskBoardTask } from '../../../types/task-board';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { textInputHandler } from '../../../utils/signal-input';

const boardQueryKey = (providerId?: string, projectId?: string) => [
  'task-board',
  providerId ?? '',
  projectId ?? ''
];

const projectsQueryKey = ['task-board-projects'];
const projectConfigsQueryKey = ['projects'];
const reposQueryKey = ['github-repos'];

const firstColumnSlug = (columns: TaskBoardColumn[]): string => columns[0]?.slug ?? '';

const targetColumnSlug = (columns: TaskBoardColumn[]): string =>
  columns.find((column) => column.isFinal)?.slug ??
  columns[columns.length - 1]?.slug ??
  firstColumnSlug(columns);

const progressColumnSlug = (columns: TaskBoardColumn[]): string =>
  columns.find((column) => !column.isFinal && column.slug !== firstColumnSlug(columns))?.slug ??
  firstColumnSlug(columns);

const matchingProjectConfig = (
  configs: ProjectConfig[],
  providerId?: string,
  externalProjectId?: string
): ProjectConfig | null =>
  configs.find(
    (config) => config.providerId === providerId && config.externalProjectId === externalProjectId
  ) ?? null;

export const useTaskBoard = () => {
  const selection = parseTaskProjectKey(selectedTaskProjectKey.value);
  const title = useSignal('');
  const body = useSignal('');
  const status = useSignal('');
  const createError = useSignal<string | null>(null);
  const repoError = useSignal<string | null>(null);
  const selectedTask = useSignal<TaskBoardTask | null>(null);
  const draggedTask = useSignal<string | null>(null);

  const boardQuery = useApiQuery(
    boardQueryKey(selection?.providerId, selection?.externalProjectId),
    () => getTaskBoard(selection?.providerId ?? '', selection?.externalProjectId ?? ''),
    { enabled: Boolean(selection) }
  );
  const { data: providerProjects = [] } = useApiQuery(projectsQueryKey, listTaskBoardProjects, {
    enabled: Boolean(selection)
  });
  const { data: projectConfigs = [] } = useApiQuery(projectConfigsQueryKey, listProjects, {
    enabled: Boolean(selection)
  });
  const { data: repos = [], isLoading: reposLoading } = useApiQuery(reposQueryKey, listRepos, {
    enabled: Boolean(selection)
  });

  const board = boardQuery.data?.board;
  const columns = board?.columns ?? [];
  const projectConfig = matchingProjectConfig(
    projectConfigs,
    selection?.providerId,
    selection?.externalProjectId
  );
  const providerProject = providerProjects.find(
    (project) =>
      project.providerId === selection?.providerId &&
      project.externalProjectId === selection?.externalProjectId
  );
  const selectedRepoNames = projectConfig?.repoFullNames ?? [];

  useEffect(() => {
    if (!columns.length) {
      status.value = '';
      return;
    }

    const statusStillExists = columns.some((column) => column.slug === status.value);
    if (!statusStillExists) {
      status.value = firstColumnSlug(columns);
    }
  }, [columns, status.value]);

  const createMutation = useApiMutation(
    () =>
      createTask(selection?.providerId ?? '', selection?.externalProjectId ?? '', {
        title: title.value.trim(),
        body: body.value,
        status: status.value || undefined
      }),
    {
      onSuccess: () => {
        title.value = '';
        body.value = '';
        createError.value = null;
        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const moveMutation = useApiMutation(
    ({ taskId, nextStatus }: { taskId: string; nextStatus: string }) =>
      moveTask(selection?.providerId ?? '', { taskId, status: nextStatus }),
    {
      onSuccess: () => {
        invalidate(...boardQueryKey(selection?.providerId, selection?.externalProjectId));
      }
    }
  );

  const repoMutation = useApiMutation(
    (nextRepoNames: string[]) => {
      if (projectConfig) {
        return updateProject(projectConfig.id, { repoFullNames: nextRepoNames });
      }

      return createProject({
        externalProjectId: selection?.externalProjectId ?? '',
        externalWorkspaceId: providerProject?.workspaceId ?? '',
        name: board?.project.name ?? providerProject?.name ?? '',
        providerId: selection?.providerId ?? '',
        enabled: true,
        pickupColumn: firstColumnSlug(columns),
        progressColumn: progressColumnSlug(columns),
        targetColumn: targetColumnSlug(columns),
        repoFullNames: nextRepoNames
      });
    },
    {
      onSuccess: () => {
        repoError.value = null;
        invalidate(...projectConfigsQueryKey);
      }
    }
  );

  const submitTask = useCallback(
    (event: Event) => {
      event.preventDefault();
      if (!title.value.trim()) {
        createError.value = 'Task title is required';
        return;
      }

      createMutation.mutate(undefined);
    },
    [createMutation, title, createError]
  );

  const selectStatus = useCallback(
    (nextStatus: string) => {
      status.value = nextStatus;
    },
    [status]
  );

  const moveTaskToStatus = useCallback(
    (taskId: string, nextStatus: string) => {
      moveMutation.mutate({ taskId, nextStatus });
    },
    [moveMutation]
  );

  const toggleRepo = useCallback(
    (repoFullName: string) => {
      if (!selection || !columns.length) {
        repoError.value = 'Board columns must load before repositories can be connected';
        return;
      }

      const nextRepoNames = selectedRepoNames.includes(repoFullName)
        ? selectedRepoNames.filter((name) => name !== repoFullName)
        : [...selectedRepoNames, repoFullName];
      repoMutation.mutate(nextRepoNames);
    },
    [selection, columns, selectedRepoNames, repoMutation, repoError]
  );

  const openTask = useCallback(
    (task: TaskBoardTask) => {
      selectedTask.value = task;
    },
    [selectedTask]
  );

  const closeTask = useCallback(() => {
    selectedTask.value = null;
  }, [selectedTask]);

  const startDrag = useCallback(
    (taskId: string) => {
      draggedTask.value = taskId;
    },
    [draggedTask]
  );

  const allowDrop = useCallback((event: DragEvent) => {
    event.preventDefault();
  }, []);

  const dropOnStatus = useCallback(
    (event: DragEvent, nextStatus: string) => {
      event.preventDefault();
      if (!draggedTask.value) return;

      moveMutation.mutate({ taskId: draggedTask.value, nextStatus });
      draggedTask.value = null;
    },
    [draggedTask, moveMutation]
  );

  return {
    data: {
      selectedProjectKey: selectedTaskProjectKey.value,
      board,
      columns,
      statusOptions: columns.map((column) => ({ value: column.slug, label: column.name })),
      repoItems: repos.map((repo) => ({ value: repo.fullName, label: repo.fullName })),
      selectedRepoNames,
      selectedTask: selectedTask.value
    },
    form: {
      title: title.value,
      body: body.value,
      status: status.value,
      createError: createError.value,
      serverError:
        createMutation.error?.message ??
        moveMutation.error?.message ??
        repoMutation.error?.message ??
        repoError.value ??
        null
    },
    status: {
      loading: boardQuery.isLoading,
      error: boardQuery.error?.message ?? null,
      creating: createMutation.isPending,
      movingTaskId: moveMutation.variables?.taskId ?? null,
      moving: moveMutation.isPending,
      reposLoading,
      connectingRepos: repoMutation.isPending,
      connected: Boolean(projectConfig)
    },
    actions: {
      onTitleInput: textInputHandler(title),
      onBodyInput: textInputHandler(body),
      onStatusChange: selectStatus,
      onSubmitTask: submitTask,
      onMoveTask: moveTaskToStatus,
      onToggleRepo: toggleRepo,
      onOpenTask: openTask,
      onCloseTask: closeTask,
      onDragStart: startDrag,
      onDragOver: allowDrop,
      onDropOnStatus: dropOnStatus
    }
  };
};
