import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardColumnPreferences, TaskBoardRepositoryItem } from '../types';
import {
  HELP_CARDS,
  hasCustomTaskBoardColumnView,
  normalizeTaskBoardColumnPreferences,
  readTaskBoardColumnPreferences,
  writeTaskBoardColumnPreferences
} from './taskBoard.helpers';
import {
  buildTaskBoardColumns,
  buildTaskBoardMoveActions,
  optionToNullableColumn,
  type UseTaskBoardViewModelInput,
  type UseTaskBoardViewModelResult
} from './taskBoardViewModel.support';

export const useTaskBoardViewModel = ({
  selectedProjectKey,
  board,
  statusOptions,
  repoItems,
  selectedRepoNames,
  selectedTask,
  augmentationsByTaskRef,
  visibleTaskCounts,
  columnRoles,
  moving,
  movingTaskId,
  actionMenuTaskId,
  actionMenuPosition,
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
  const columnPreferences = useSignal<TaskBoardColumnPreferences>(
    readTaskBoardColumnPreferences(selectedProjectKey)
  );

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

  useEffect(() => {
    columnPreferences.value = readTaskBoardColumnPreferences(selectedProjectKey);
  }, [selectedProjectKey]);

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
  const setReviewColumn = useCallback(
    (value: string) => {
      onSetColumnRole(optionToNullableColumn(value), 'review');
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
  const normalizedColumnPreferences = normalizeTaskBoardColumnPreferences(
    boardColumns,
    columnPreferences.value
  );
  const orderedBoardColumns = normalizedColumnPreferences.columnOrder
    .map((columnSlug) => boardColumns.find((column) => column.slug === columnSlug))
    .filter((column): column is NonNullable<typeof column> => Boolean(column));
  const hiddenColumnSlugSet = new Set(normalizedColumnPreferences.hiddenColumnSlugs);
  const visibleBoardColumns = orderedBoardColumns.filter(
    (column) => !hiddenColumnSlugSet.has(column.slug)
  );

  const saveColumnPreferences = useCallback(
    (preferences: TaskBoardColumnPreferences) => {
      const nextPreferences = normalizeTaskBoardColumnPreferences(boardColumns, preferences);
      const storedPreferences = hasCustomTaskBoardColumnView(boardColumns, nextPreferences)
        ? nextPreferences
        : { hiddenColumnSlugs: [], columnOrder: [] };
      columnPreferences.value = storedPreferences;
      writeTaskBoardColumnPreferences(selectedProjectKey, storedPreferences);
    },
    [boardColumns, columnPreferences, selectedProjectKey]
  );

  const showColumn = useCallback(
    (columnSlug: string) => {
      saveColumnPreferences({
        ...normalizedColumnPreferences,
        hiddenColumnSlugs: normalizedColumnPreferences.hiddenColumnSlugs.filter(
          (hiddenColumnSlug) => hiddenColumnSlug !== columnSlug
        )
      });
    },
    [normalizedColumnPreferences, saveColumnPreferences]
  );

  const hideColumn = useCallback(
    (columnSlug: string) => {
      if (normalizedColumnPreferences.hiddenColumnSlugs.includes(columnSlug)) {
        return;
      }

      saveColumnPreferences({
        ...normalizedColumnPreferences,
        hiddenColumnSlugs: [...normalizedColumnPreferences.hiddenColumnSlugs, columnSlug]
      });
    },
    [normalizedColumnPreferences, saveColumnPreferences]
  );

  const moveColumn = useCallback(
    (columnSlug: string, offset: -1 | 1) => {
      const visibleColumnSlugs = visibleBoardColumns.map((column) => column.slug);
      const visibleIndex = visibleColumnSlugs.indexOf(columnSlug);
      const targetSlug = visibleColumnSlugs[visibleIndex + offset];

      if (visibleIndex < 0 || !targetSlug) {
        return;
      }

      const columnOrder = [...normalizedColumnPreferences.columnOrder];
      const currentIndex = columnOrder.indexOf(columnSlug);
      const targetIndex = columnOrder.indexOf(targetSlug);

      if (currentIndex < 0 || targetIndex < 0) {
        return;
      }

      columnOrder[currentIndex] = targetSlug;
      columnOrder[targetIndex] = columnSlug;
      saveColumnPreferences({ ...normalizedColumnPreferences, columnOrder });
    },
    [normalizedColumnPreferences, saveColumnPreferences, visibleBoardColumns]
  );

  const moveColumnLeft = useCallback(
    (columnSlug: string) => {
      moveColumn(columnSlug, -1);
    },
    [moveColumn]
  );
  const moveColumnRight = useCallback(
    (columnSlug: string) => {
      moveColumn(columnSlug, 1);
    },
    [moveColumn]
  );
  const resetColumnView = useCallback(() => {
    columnPreferences.value = { hiddenColumnSlugs: [], columnOrder: [] };
    writeTaskBoardColumnPreferences(selectedProjectKey, { hiddenColumnSlugs: [], columnOrder: [] });
  }, [columnPreferences, selectedProjectKey]);

  const hiddenColumns = orderedBoardColumns
    .filter((column) => hiddenColumnSlugSet.has(column.slug))
    .map((column) => ({
      column,
      onShow: () => showColumn(column.slug)
    }));
  const columns = buildTaskBoardColumns({
    boardColumns: visibleBoardColumns,
    statusOptions,
    augmentationsByTaskRef,
    visibleTaskCounts,
    columnRoles,
    moving,
    movingTaskId,
    actionMenuTaskId,
    actionMenuPosition,
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
    onSetColumnRole,
    onHideColumn: hideColumn,
    onMoveColumnLeft: moveColumnLeft,
    onMoveColumnRight: moveColumnRight
  });

  return {
    data: {
      boardColumnCount: Math.max(visibleBoardColumns.length, 1),
      columns,
      hiddenColumns,
      hasCustomColumnView: hasCustomTaskBoardColumnView(boardColumns, columnPreferences.value),
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
            id: 'board-settings-review-column',
            label: 'Review column',
            value: columnRoles.reviewColumn,
            options: statusOptions,
            onValueChange: setReviewColumn
          },
          {
            id: 'board-settings-done-column',
            label: 'Done column',
            value: columnRoles.doneColumn,
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
      selectedTaskMoveActions,
      selectedTaskAugmentation: selectedTask
        ? (augmentationsByTaskRef.get(selectedTask.id) ?? null)
        : null
    },
    actions: {
      onFilterRepos: filterRepos,
      onPickupColumnChange: setPickupColumn,
      onProgressColumnChange: setProgressColumn,
      onReviewColumnChange: setReviewColumn,
      onDoneColumnChange: setDoneColumn,
      onShowColumn: showColumn,
      onHideColumn: hideColumn,
      onMoveColumnLeft: moveColumnLeft,
      onMoveColumnRight: moveColumnRight,
      onResetColumnView: resetColumnView
    }
  };
};
