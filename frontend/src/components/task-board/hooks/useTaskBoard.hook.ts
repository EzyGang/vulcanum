import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import {
  DEFAULT_MAX_IN_PROGRESS_TASKS,
  DEFAULT_REVIEW_MAX_TURNS
} from '../../../constants/reviewAutomation';
import { listRepos } from '../../../services/github/github.service';
import { listProjects, updateProject } from '../../../services/projects/projects.service';
import {
  createTask,
  getTaskBoard,
  moveTask
} from '../../../services/task-board/task-board.service';
import { parseTaskProjectKey, selectedTaskProjectKey } from '../../../stores/task-board.store';
import type { ProjectConfig, UpdateProjectRequest } from '../../../types/projects';
import type { TaskBoardColumn, TaskBoardTask } from '../../../types/task-board';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { parsePositiveNumber } from '../../../utils/numbers';
import { textInputHandler } from '../../../utils/signal-input';
import type {
  TaskBoardColumnRole,
  TaskBoardColumnRoles,
  TaskBoardSettingsFormState
} from '../types';

const boardQueryKey = (providerId?: string, projectId?: string) => [
  'task-board',
  providerId ?? '',
  projectId ?? ''
];

const projectConfigsQueryKey = ['projects'];
const reposQueryKey = ['github-repos'];
const taskBoardProjectsQueryKey = ['task-board-projects'];
const COLUMN_PAGE_SIZE = 20;

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

const columnRolesForProject = (
  config: ProjectConfig | null,
  columns: TaskBoardColumn[]
): TaskBoardColumnRoles => ({
  pickupColumn: config?.pickupColumn || firstColumnSlug(columns),
  progressColumn: config?.progressColumn || progressColumnSlug(columns),
  targetColumn: config?.targetColumn || targetColumnSlug(columns),
  reviewPickupColumn: config?.reviewPickupColumn ?? null
});

const nullableText = (value: string): string | null => {
  const trimmed = value.trim();
  return trimmed.length > 0 ? value : null;
};

const settingsFormFromConfig = (config: ProjectConfig | null): TaskBoardSettingsFormState => ({
  promptTemplate: config?.promptTemplate ?? '',
  agentsMd: config?.agentsMd ?? '',
  reviewEnabled:
    config?.reviewEnabled === null || config?.reviewEnabled === undefined
      ? ''
      : config.reviewEnabled
        ? 'true'
        : 'false',
  reviewPickupColumn: config?.reviewPickupColumn ?? '',
  reviewMaxTurns: config?.reviewMaxTurns?.toString() ?? '',
  reviewPromptTemplate: config?.reviewPromptTemplate ?? '',
  maxInProgressTasks: config?.maxInProgressTasks?.toString() ?? ''
});

export const useTaskBoard = () => {
  const selection = parseTaskProjectKey(selectedTaskProjectKey.value);
  const title = useSignal('');
  const body = useSignal('');
  const status = useSignal('');
  const createError = useSignal<string | null>(null);
  const formError = useSignal<string | null>(null);
  const selectedTask = useSignal<TaskBoardTask | null>(null);
  const draggedTask = useSignal<string | null>(null);
  const createDialogOpen = useSignal(false);
  const settingsDialogOpen = useSignal(false);
  const actionMenuTaskId = useSignal<string | null>(null);
  const visibleTaskCounts = useSignal<Record<string, number>>({});
  const settingsPromptTemplate = useSignal('');
  const settingsAgentsMd = useSignal('');
  const settingsReviewEnabled = useSignal('');
  const settingsReviewPickupColumn = useSignal('');
  const settingsReviewMaxTurns = useSignal('');
  const settingsReviewPromptTemplate = useSignal('');
  const settingsMaxInProgressTasks = useSignal('');

  const boardQuery = useApiQuery(
    boardQueryKey(selection?.providerId, selection?.externalProjectId),
    () => getTaskBoard(selection?.providerId ?? '', selection?.externalProjectId ?? ''),
    {
      enabled: Boolean(selection),
      refetchInterval: 120_000,
      staleTime: 60_000
    }
  );
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
  const selectedRepoNames = projectConfig?.repoFullNames ?? [];
  const columnRoles = columnRolesForProject(projectConfig, columns);

  useEffect(() => {
    const nextCounts: Record<string, number> = {};

    for (const column of columns) {
      nextCounts[column.slug] = visibleTaskCounts.value[column.slug] ?? COLUMN_PAGE_SIZE;
    }

    visibleTaskCounts.value = nextCounts;
  }, [columns]);

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

  useEffect(() => {
    if (!settingsDialogOpen.value) return;

    const nextSettings = settingsFormFromConfig(projectConfig);
    settingsPromptTemplate.value = nextSettings.promptTemplate;
    settingsAgentsMd.value = nextSettings.agentsMd;
    settingsReviewEnabled.value = nextSettings.reviewEnabled;
    settingsReviewPickupColumn.value = nextSettings.reviewPickupColumn;
    settingsReviewMaxTurns.value = nextSettings.reviewMaxTurns;
    settingsReviewPromptTemplate.value = nextSettings.reviewPromptTemplate;
    settingsMaxInProgressTasks.value = nextSettings.maxInProgressTasks;
  }, [settingsDialogOpen.value, projectConfig?.id]);

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
        createDialogOpen.value = false;
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
      if (!projectConfig) {
        throw new Error('Add this provider project before pinning repositories');
      }

      return updateProject(projectConfig.id, { repoFullNames: nextRepoNames });
    },
    {
      onSuccess: () => {
        formError.value = null;
        invalidate(...projectConfigsQueryKey);
      }
    }
  );

  const settingsMutation = useApiMutation(
    (input: UpdateProjectRequest) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before editing board settings');
      }

      return updateProject(projectConfig.id, input);
    },
    {
      onSuccess: () => {
        formError.value = null;
        settingsDialogOpen.value = false;
        invalidate(...projectConfigsQueryKey);
        invalidate(...taskBoardProjectsQueryKey);
      }
    }
  );

  const columnRoleMutation = useApiMutation(
    (input: UpdateProjectRequest) => {
      if (!projectConfig) {
        throw new Error('Add this provider project before editing column roles');
      }

      return updateProject(projectConfig.id, input);
    },
    {
      onSuccess: () => {
        formError.value = null;
        invalidate(...projectConfigsQueryKey);
        invalidate(...taskBoardProjectsQueryKey);
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

  const submitSettings = useCallback(
    async (event: Event) => {
      event.preventDefault();
      formError.value = null;

      try {
        await settingsMutation.mutateAsync({
          promptTemplate: nullableText(settingsPromptTemplate.value),
          agentsMd: nullableText(settingsAgentsMd.value),
          reviewEnabled:
            settingsReviewEnabled.value === '' ? null : settingsReviewEnabled.value === 'true',
          reviewPickupColumn: settingsReviewPickupColumn.value || null,
          reviewMaxTurns: settingsReviewMaxTurns.value.trim()
            ? parsePositiveNumber(settingsReviewMaxTurns.value, DEFAULT_REVIEW_MAX_TURNS)
            : null,
          reviewPromptTemplate: nullableText(settingsReviewPromptTemplate.value),
          maxInProgressTasks: settingsMaxInProgressTasks.value.trim()
            ? parsePositiveNumber(settingsMaxInProgressTasks.value, DEFAULT_MAX_IN_PROGRESS_TASKS)
            : null
        });
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save board settings';
      }
    },
    [
      formError,
      settingsMutation,
      settingsPromptTemplate,
      settingsAgentsMd,
      settingsReviewEnabled,
      settingsReviewPickupColumn,
      settingsReviewMaxTurns,
      settingsReviewPromptTemplate,
      settingsMaxInProgressTasks
    ]
  );

  const setColumnRole = useCallback(
    (columnSlug: string, role: TaskBoardColumnRole) => {
      formError.value = null;
      const input: UpdateProjectRequest = {};

      if (role === 'pickup') {
        input.pickupColumn = columnSlug;
      } else if (role === 'progress') {
        input.progressColumn = columnSlug;
      } else if (role === 'done') {
        input.targetColumn = columnSlug;
      } else {
        input.reviewPickupColumn =
          projectConfig?.reviewPickupColumn === columnSlug ? null : columnSlug;
      }

      columnRoleMutation.mutate(input, {
        onError: (err) => {
          formError.value = err.message;
        }
      });
    },
    [columnRoleMutation, formError, projectConfig?.reviewPickupColumn]
  );

  const selectStatus = useCallback(
    (nextStatus: string) => {
      status.value = nextStatus;
    },
    [status]
  );

  const moveTaskToStatus = useCallback(
    (taskId: string, nextStatus: string) => {
      actionMenuTaskId.value = null;
      moveMutation.mutate({ taskId, nextStatus });
    },
    [actionMenuTaskId, moveMutation]
  );

  const toggleRepo = useCallback(
    (repoFullName: string) => {
      if (!selection || !columns.length) {
        formError.value = 'Board columns must load before repositories can be connected';
        return;
      }

      const nextRepoNames = selectedRepoNames.includes(repoFullName)
        ? selectedRepoNames.filter((name) => name !== repoFullName)
        : [...selectedRepoNames, repoFullName];
      repoMutation.mutate(nextRepoNames, {
        onError: (err) => {
          formError.value = err.message;
        }
      });
    },
    [selection, columns, selectedRepoNames, repoMutation, formError]
  );

  const openTask = useCallback(
    (task: TaskBoardTask) => {
      actionMenuTaskId.value = null;
      selectedTask.value = task;
    },
    [actionMenuTaskId, selectedTask]
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

  const openTaskMenu = useCallback(
    (event: MouseEvent, taskId: string) => {
      event.preventDefault();
      event.stopPropagation();
      actionMenuTaskId.value = actionMenuTaskId.value === taskId ? null : taskId;
    },
    [actionMenuTaskId]
  );

  const closeTaskMenu = useCallback(() => {
    actionMenuTaskId.value = null;
  }, [actionMenuTaskId]);

  const loadMoreColumn = useCallback(
    (columnSlug: string) => {
      visibleTaskCounts.value = {
        ...visibleTaskCounts.value,
        [columnSlug]: (visibleTaskCounts.value[columnSlug] ?? COLUMN_PAGE_SIZE) + COLUMN_PAGE_SIZE
      };
    },
    [visibleTaskCounts]
  );

  const scrollColumn = useCallback(
    (event: Event, columnSlug: string) => {
      const target = event.currentTarget as HTMLElement;
      const nearBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 32;

      if (nearBottom) {
        loadMoreColumn(columnSlug);
      }
    },
    [loadMoreColumn]
  );

  return {
    data: {
      selectedProjectKey: selectedTaskProjectKey.value,
      board,
      columns,
      statusOptions: columns.map((column) => ({ value: column.slug, label: column.name })),
      repoItems: repos.map((repo) => ({ value: repo.fullName, label: repo.fullName })),
      selectedRepoNames,
      selectedTask: selectedTask.value,
      createDialogOpen: createDialogOpen.value,
      settingsDialogOpen: settingsDialogOpen.value,
      actionMenuTaskId: actionMenuTaskId.value,
      visibleTaskCounts: visibleTaskCounts.value,
      columnRoles
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
        settingsMutation.error?.message ??
        columnRoleMutation.error?.message ??
        formError.value ??
        null,
      settings: {
        promptTemplate: settingsPromptTemplate.value,
        agentsMd: settingsAgentsMd.value,
        reviewEnabled: settingsReviewEnabled.value,
        reviewPickupColumn: settingsReviewPickupColumn.value,
        reviewMaxTurns: settingsReviewMaxTurns.value,
        reviewPromptTemplate: settingsReviewPromptTemplate.value,
        maxInProgressTasks: settingsMaxInProgressTasks.value
      }
    },
    status: {
      loading: boardQuery.isLoading,
      error: boardQuery.error?.message ?? null,
      creating: createMutation.isPending,
      movingTaskId: moveMutation.variables?.taskId ?? null,
      moving: moveMutation.isPending,
      reposLoading,
      connectingRepos: repoMutation.isPending,
      connected: Boolean(projectConfig),
      savingSettings: settingsMutation.isPending,
      configuringColumns: columnRoleMutation.isPending
    },
    actions: {
      onTitleInput: textInputHandler(title),
      onBodyInput: textInputHandler(body),
      onStatusChange: selectStatus,
      onSubmitTask: submitTask,
      onMoveTask: moveTaskToStatus,
      onToggleRepo: toggleRepo,
      onSettingsPromptInput: textInputHandler(settingsPromptTemplate),
      onSettingsAgentsInput: textInputHandler(settingsAgentsMd),
      onSettingsReviewEnabledChange: (value: string) => {
        settingsReviewEnabled.value = value;
      },
      onSettingsReviewPickupColumnChange: (value: string) => {
        settingsReviewPickupColumn.value = value;
      },
      onSettingsReviewMaxTurnsInput: textInputHandler(settingsReviewMaxTurns),
      onSettingsReviewPromptInput: textInputHandler(settingsReviewPromptTemplate),
      onSettingsMaxInProgressInput: textInputHandler(settingsMaxInProgressTasks),
      onSubmitSettings: submitSettings,
      onSetColumnRole: setColumnRole,
      onOpenTask: openTask,
      onCloseTask: closeTask,
      onDragStart: startDrag,
      onDragOver: allowDrop,
      onDropOnStatus: dropOnStatus,
      onOpenCreateTask: () => {
        createDialogOpen.value = true;
      },
      onCloseCreateTask: () => {
        createDialogOpen.value = false;
      },
      onOpenSettings: () => {
        formError.value = null;
        settingsDialogOpen.value = true;
      },
      onCloseSettings: () => {
        settingsDialogOpen.value = false;
      },
      onOpenTaskMenu: openTaskMenu,
      onCloseTaskMenu: closeTaskMenu,
      onLoadMoreColumn: loadMoreColumn,
      onColumnScroll: scrollColumn
    }
  };
};
