import { useMemo } from 'preact/hooks';
import { listRepos } from '../../../services/github/github.service';
import { listProjects } from '../../../services/projects/projects.service';
import { getTaskBoard } from '../../../services/task-board/task-board.service';
import { parseTaskProjectKey, selectedTaskProjectKey } from '../../../stores/task-board.store';
import { useApiQuery } from '../../../utils/api/query/hooks';
import {
  boardQueryKey,
  columnRolesForProject,
  matchingProjectConfig,
  projectConfigsQueryKey,
  reposQueryKey
} from './taskBoard.helpers';
import { useTaskBoardCreate } from './useTaskBoardCreate.hook';
import { useTaskBoardMovement } from './useTaskBoardMovement.hook';
import { useTaskBoardSettings } from './useTaskBoardSettings.hook';
import { useTaskBoardViewModel } from './useTaskBoardViewModel.hook';

export const useTaskBoard = () => {
  const selection = parseTaskProjectKey(selectedTaskProjectKey.value);
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
  const statusOptions = columns.map((column) => ({ value: column.slug, label: column.name }));
  const repoItems = repos.map((repo) => ({ value: repo.fullName, label: repo.fullName }));
  const relatedRunsByTaskRef = useMemo(
    () =>
      new Map(
        (boardQuery.data?.relatedTaskRuns ?? []).map((item) => [item.externalTaskRef, item.runs])
      ),
    [boardQuery.data?.relatedTaskRuns]
  );

  const create = useTaskBoardCreate(selection, columns);
  const movement = useTaskBoardMovement(selection, columns, board?.labels ?? []);
  const settings = useTaskBoardSettings(selection, columns, projectConfig, selectedRepoNames);
  const viewModel = useTaskBoardViewModel({
    board,
    statusOptions,
    repoItems,
    selectedRepoNames,
    selectedTask: movement.data.selectedTask,
    relatedRunsByTaskRef,
    visibleTaskCounts: movement.data.visibleTaskCounts,
    columnRoles,
    moving: movement.status.moving,
    movingTaskId: movement.status.movingTaskId,
    actionMenuTaskId: movement.data.actionMenuTaskId,
    actionMenuPosition: movement.data.actionMenuPosition,
    configuringColumns: settings.status.configuringColumns,
    dropPreviewColumn: movement.data.dropPreviewColumn,
    automationEnabled: settings.data.automationEnabled,
    dismissedHelpCards: settings.data.dismissedHelpCards,
    settingsForm: settings.form,
    onMoveTask: movement.actions.onMoveTask,
    onOpenTask: movement.actions.onOpenTask,
    onOpenTaskMenu: movement.actions.onOpenTaskMenu,
    onDragStart: movement.actions.onDragStart,
    onDragOverStatus: movement.actions.onDragOverStatus,
    onDragEnd: movement.actions.onDragEnd,
    onDropOnStatus: movement.actions.onDropOnStatus,
    onLoadMoreColumn: movement.actions.onLoadMoreColumn,
    onColumnScroll: movement.actions.onColumnScroll,
    onSetColumnRole: settings.actions.onSetColumnRole,
    onToggleRepo: settings.actions.onToggleRepo,
    onDismissHelpCard: settings.actions.onDismissHelpCard
  });

  return {
    data: {
      selectedProjectKey: selectedTaskProjectKey.value,
      board,
      ...viewModel.data,
      statusOptions,
      repoItems,
      selectedRepoNames,
      selectedTask: movement.data.selectedTask,
      availableLabels: board?.labels ?? [],
      createDialogOpen: create.dialogOpen,
      settingsDialogOpen: settings.data.settingsDialogOpen,
      actionMenuTaskId: movement.data.actionMenuTaskId,
      actionMenuPosition: movement.data.actionMenuPosition,
      visibleTaskCounts: movement.data.visibleTaskCounts,
      columnRoles,
      dropPreviewColumn: movement.data.dropPreviewColumn,
      automationEnabled: settings.data.automationEnabled,
      dismissedHelpCards: settings.data.dismissedHelpCards
    },
    form: {
      title: create.form.title,
      body: create.form.body,
      status: create.form.status,
      createError: create.form.createError,
      ...movement.form,
      serverError: create.error ?? movement.error ?? settings.error,
      settings: settings.form
    },
    status: {
      loading: boardQuery.isLoading,
      error: boardQuery.error?.message ?? null,
      creating: create.status.creating,
      movingTaskId: movement.status.movingTaskId,
      moving: movement.status.moving,
      updatingTask: movement.status.updatingTask,
      updatingTaskLabel: movement.status.updatingTaskLabel,
      reposLoading,
      ...settings.status
    },
    actions: {
      ...create.actions,
      ...movement.actions,
      ...settings.actions,
      ...viewModel.actions
    }
  };
};
