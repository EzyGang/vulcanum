import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import type { SelectOption } from '../../../types/shared';
import type { TaskBoardTask } from '../../../types/task-board';
import type { TaskBoardColumnData, TaskBoardMoveAction, TaskBoardRepositoryItem } from '../types';
import { HELP_CARDS } from './taskBoard.helpers';
import {
  columnRoleActive,
  formatCreatedAt,
  optionToNullableColumn,
  ROLE_HELP,
  ROLE_LABELS,
  ROLE_ORDER,
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
  }, [openRoleMenuColumn, openRoleMenuColumn.value]);

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
  const setReviewColumn = useCallback(
    (value: string) => {
      onSetColumnRole(optionToNullableColumn(value), 'review');
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

  const createMoveActions = (task: TaskBoardTask): TaskBoardMoveAction[] =>
    statusOptions
      .filter((option) => option.value !== task.status)
      .map((option) => ({
        value: option.value,
        label: option.label,
        onClick: (event) => {
          event.stopPropagation();
          onMoveTask(task.id, option.value);
        }
      }));

  const selectedTaskMoveActions = selectedTask ? createMoveActions(selectedTask) : [];
  const boardColumns = board?.columns ?? [];
  const columns = boardColumns.map((column): TaskBoardColumnData => {
    const visibleCount = visibleTaskCounts[column.slug] ?? 20;
    const visibleTasks = column.tasks.slice(0, visibleCount);
    const activeRoles = ROLE_ORDER.filter((role) =>
      columnRoleActive(role, column.slug, columnRoles)
    );
    const dropPreviewActive = dropPreviewColumn === column.slug;

    return {
      column,
      visibleTasks: visibleTasks.map((task) => ({
        task,
        displayId: task.number ? `#${task.number}` : task.id.slice(0, 8),
        createdAtLabel: formatCreatedAt(task.createdAt),
        moving: moving && movingTaskId === task.id,
        menuOpen: actionMenuTaskId === task.id,
        moveActions: createMoveActions(task),
        onClick: () => onOpenTask(task),
        onContextMenu: (event) => onOpenTaskMenu(event as unknown as MouseEvent, task.id),
        onDragStart: () => onDragStart(task.id, task.status),
        onDragEnd,
        onKeyDown: (event) => {
          if (event.key !== 'Enter' && event.key !== ' ') return;

          event.preventDefault();
          onOpenTask(task);
        },
        onStopMenuClick: (event) => {
          event.stopPropagation();
        }
      })),
      taskCount: column.tasks.length,
      activeRoles: activeRoles.map((role) => ({ role })),
      hasMoreTasks: visibleTasks.length < column.tasks.length,
      dropPreviewActive,
      roleMenu: {
        buttonLabel: `Column role settings for ${column.name}`,
        menuLabel: `Column roles for ${column.name}`,
        open: openRoleMenuColumn.value === column.slug,
        disabled: configuringColumns,
        onToggle: (event) => {
          event.preventDefault();
          event.stopPropagation();
          openRoleMenuColumn.value = openRoleMenuColumn.value === column.slug ? null : column.slug;
        },
        onStopClick: (event) => {
          event.stopPropagation();
        },
        items: ROLE_ORDER.map((role) => {
          const active = activeRoles.includes(role);
          const disabled = configuringColumns || (active && role !== 'review');

          return {
            role,
            label: `${active && role === 'review' ? 'Clear' : 'Set'} ${ROLE_LABELS[role]}`,
            help: ROLE_HELP[role],
            active,
            disabled,
            onClick: (event) => {
              event.preventDefault();
              event.stopPropagation();
              const clearRole = role === 'review' && active;
              onSetColumnRole(clearRole ? null : column.slug, role);
              openRoleMenuColumn.value = null;
            }
          };
        })
      },
      onDragOver: (event) => onDragOverStatus(event as unknown as DragEvent, column.slug),
      onDrop: (event) => onDropOnStatus(event as unknown as DragEvent, column.slug),
      onScroll: (event) => onColumnScroll(event, column.slug),
      onLoadMore: () => onLoadMoreColumn(column.slug)
    };
  });

  const reviewPickupColumnItems = [
    { value: '', label: 'Use column role or team default' },
    ...statusOptions
  ];

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
        hasOverrides: columnRoles.reviewPickupColumn !== null,
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
          },
          {
            id: 'board-settings-review-pickup-column',
            label: 'Review pickup column',
            value: columnRoles.reviewPickupColumn ?? '',
            options: [{ value: '', label: 'No review pickup override' }, ...statusOptions],
            onValueChange: setReviewColumn
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
        reviewPickupColumnItems,
        hasOverrides:
          settingsForm.reviewEnabled !== '' ||
          settingsForm.reviewPickupColumn !== '' ||
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
      onDoneColumnChange: setDoneColumn,
      onReviewColumnChange: setReviewColumn
    }
  };
};
