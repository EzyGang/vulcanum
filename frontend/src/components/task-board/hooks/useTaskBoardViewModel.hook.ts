import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardRepositoryItem } from '../types';
import { HELP_CARDS } from './taskBoard.helpers';
import {
  buildTaskBoardColumns,
  buildTaskBoardMoveActions,
  optionToNullableColumn,
  type UseTaskBoardViewModelInput,
  type UseTaskBoardViewModelResult
} from './taskBoardViewModel.support';

export const useTaskBoardViewModel = ({
  board,
  statusOptions,
  repoItems,
  selectedRepoNames,
  selectedTask,
  visibleTaskCounts,
  columnRoles,
  moving,
  movingTaskId,
  actionMenuTaskId,
  configuringColumns,
  dropPreviewColumn,
  automationEnabled,
  dismissedHelpCards,
  settingsForm,
  onMoveTask,
  onOpenTask,
  onOpenTaskMenu,
  onDragStart,
  onDragOverStatus,
  onDragEnd,
  onDropOnStatus,
  onLoadMoreColumn,
  onColumnScroll,
  onSetColumnRole,
  onToggleRepo,
  onDismissHelpCard
}: UseTaskBoardViewModelInput): UseTaskBoardViewModelResult => {
  const repoFilter = useSignal('');
  const openRoleMenuColumn = useSignal<string | null>(null);

  useEffect(() => {
    if (openRoleMenuColumn.value === null) return;

    const closeRoleMenu = () => {
      openRoleMenuColumn.value = null;
    };

    window.addEventListener('click', closeRoleMenu);

    return () => {
      window.removeEventListener('click', closeRoleMenu);
    };
  }, [openRoleMenuColumn.value]);

  const filterRepos = useCallback(
    (event: Event) => {
      repoFilter.value = (event.target as HTMLInputElement).value;
    },
    [repoFilter]
  );

  const setPickupColumn = useCallback(
    (value: string) => {
      onSetColumnRole(optionToNullableColumn(value), 'pickup');
    },
    [onSetColumnRole]
  );
  const setProgressColumn = useCallback(
    (value: string) => {
      onSetColumnRole(optionToNullableColumn(value), 'progress');
    },
    [onSetColumnRole]
  );
  const setDoneColumn = useCallback(
    (value: string) => {
      onSetColumnRole(optionToNullableColumn(value), 'done');
    },
    [onSetColumnRole]
  );

  const selectedRepoNameSet = new Set(selectedRepoNames);
  const makeRepoItem = (repo: SelectOption, checked: boolean): TaskBoardRepositoryItem => ({
    ...repo,
    checked,
    onToggle: () => onToggleRepo(repo.value)
  });
  const selectedRepos = selectedRepoNames.map((repoFullName) =>
    makeRepoItem(
      repoItems.find((repo) => repo.value === repoFullName) ?? {
        value: repoFullName,
        label: repoFullName
      },
      true
    )
  );
  const normalizedRepoFilter = repoFilter.value.trim().toLocaleLowerCase();
  const filteredRepos = repoItems
    .filter(
      (repo) =>
        !selectedRepoNameSet.has(repo.value) &&
        (normalizedRepoFilter.length === 0 ||
          repo.label.toLocaleLowerCase().includes(normalizedRepoFilter) ||
          repo.value.toLocaleLowerCase().includes(normalizedRepoFilter))
    )
    .map((repo) => makeRepoItem(repo, false));

  const selectedTaskMoveActions = selectedTask
    ? buildTaskBoardMoveActions(selectedTask, statusOptions, onMoveTask)
    : [];
  const boardColumns = board?.columns ?? [];
  const columns = buildTaskBoardColumns({
    boardColumns,
    statusOptions,
    visibleTaskCounts,
    columnRoles,
    moving,
    movingTaskId,
    actionMenuTaskId,
    configuringColumns,
    dropPreviewColumn,
    openRoleMenuColumn: openRoleMenuColumn.value,
    onRoleMenuColumnChange: (columnSlug) => {
      openRoleMenuColumn.value = columnSlug;
    },
    onMoveTask,
    onOpenTask,
    onOpenTaskMenu,
    onDragStart,
    onDragOverStatus,
    onDragEnd,
    onDropOnStatus,
    onLoadMoreColumn,
    onColumnScroll,
    onSetColumnRole
  });

  return {
    data: {
      boardColumnCount: Math.max(boardColumns.length, 1),
      columns,
      helpCards: HELP_CARDS.filter((card) => !dismissedHelpCards.includes(card.id)).map((card) => ({
        ...card,
        onDismiss: () => onDismissHelpCard(card.id)
      })),
      automationLabel: automationEnabled ? 'Automation on' : 'Automation off',
      repositorySettings: {
        filter: repoFilter.value,
        selectedRepos,
        filteredRepos,
        hasRepos: repoItems.length > 0,
        hasSelectedRepos: selectedRepos.length > 0,
        hasFilteredRepos: filteredRepos.length > 0,
        hasOverrides: selectedRepoNames.length > 0
      },
      columnSettings: {
        hasOptions: statusOptions.length > 0,
        roleSelects: [
          {
            id: 'board-settings-pickup-column',
            label: 'Pickup column',
            value: columnRoles.pickupColumn,
            options: statusOptions,
            onValueChange: setPickupColumn
          },
          {
            id: 'board-settings-progress-column',
            label: 'In-progress column',
            value: columnRoles.progressColumn,
            options: statusOptions,
            onValueChange: setProgressColumn
          },
          {
            id: 'board-settings-done-column',
            label: 'Done column',
            value: columnRoles.targetColumn,
            options: statusOptions,
            onValueChange: setDoneColumn
          }
        ]
      },
      projectSettings: {
        hasOverrides:
          settingsForm.promptTemplate.trim().length > 0 ||
          settingsForm.agentsMd.trim().length > 0 ||
          settingsForm.maxInProgressTasks.trim().length > 0
      },
      reviewSettings: {
        hasOverrides:
          settingsForm.reviewEnabled !== '' ||
          settingsForm.reviewMaxTurns.trim().length > 0 ||
          settingsForm.reviewPromptTemplate.trim().length > 0
      },
      selectedTaskCreatedAtLabel: selectedTask
        ? new Date(selectedTask.createdAt).toLocaleString()
        : null,
      selectedTaskMoveActions
    },
    actions: {
      onFilterRepos: filterRepos,
      onPickupColumnChange: setPickupColumn,
      onProgressColumnChange: setProgressColumn,
      onDoneColumnChange: setDoneColumn
    }
  };
};
