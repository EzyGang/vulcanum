import type { SelectOption } from '../../../types/shared';
import type {
  TaskBoard,
  TaskBoardTask,
  TaskBoardTaskAugmentation
} from '../../../types/task-board';
import type {
  TaskBoardColumnData,
  TaskBoardColumnRole,
  TaskBoardColumnRoles,
  TaskBoardHelpCard,
  TaskBoardHiddenColumnData,
  TaskBoardMenuPosition,
  TaskBoardMenuStyle,
  TaskBoardMoveAction,
  TaskBoardProjectSettingsData,
  TaskBoardRepositorySettingsData,
  TaskBoardRoleSelectData,
  TaskBoardSettingsFormState
} from '../types';

export interface UseTaskBoardViewModelInput {
  selectedProjectKey: string | null;
  board?: TaskBoard;
  statusOptions: SelectOption[];
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  selectedTask: TaskBoardTask | null;
  augmentationsByTaskRef: ReadonlyMap<string, TaskBoardTaskAugmentation>;
  visibleTaskCounts: Record<string, number>;
  columnRoles: TaskBoardColumnRoles;
  moving: boolean;
  movingTaskId: string | null;
  actionMenuTaskId: string | null;
  actionMenuPosition: TaskBoardMenuPosition | null;
  configuringColumns: boolean;
  dropPreviewColumn: string | null;
  automationEnabled: boolean;
  dismissedHelpCards: TaskBoardHelpCard[];
  settingsForm: TaskBoardSettingsFormState;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onDragStart: (taskId: string, status: string) => void;
  onDragOverStatus: (event: DragEvent, status: string) => void;
  onDragEnd: () => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onLoadMoreColumn: (columnSlug: string) => void;
  onColumnScroll: (event: Event, columnSlug: string) => void;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
  onToggleRepo: (repoFullName: string) => void;
  onDismissHelpCard: (card: TaskBoardHelpCard) => void;
}

export interface UseTaskBoardViewModelResult {
  data: {
    boardColumnCount: number;
    columns: TaskBoardColumnData[];
    hiddenColumns: TaskBoardHiddenColumnData[];
    hasCustomColumnView: boolean;
    helpCards: {
      id: TaskBoardHelpCard;
      title: string;
      body: string;
      onDismiss: () => void;
    }[];
    automationLabel: string;
    repositorySettings: TaskBoardRepositorySettingsData;
    columnSettings: {
      hasOptions: boolean;
      roleSelects: TaskBoardRoleSelectData[];
    };
    projectSettings: TaskBoardProjectSettingsData;
    reviewSettings: { hasOverrides: boolean };
    selectedTaskCreatedAtLabel: string | null;
    selectedTaskMoveActions: TaskBoardMoveAction[];
    selectedTaskAugmentation: TaskBoardTaskAugmentation | null;
  };
  actions: {
    onFilterRepos: (event: Event) => void;
    onPickupColumnChange: (value: string) => void;
    onProgressColumnChange: (value: string) => void;
    onReviewColumnChange: (value: string) => void;
    onDoneColumnChange: (value: string) => void;
    onShowColumn: (columnSlug: string) => void;
    onHideColumn: (columnSlug: string) => void;
    onMoveColumnLeft: (columnSlug: string) => void;
    onMoveColumnRight: (columnSlug: string) => void;
    onResetColumnView: () => void;
  };
}

interface BuildTaskBoardColumnsInput {
  boardColumns: TaskBoard['columns'];
  statusOptions: SelectOption[];
  visibleTaskCounts: Record<string, number>;
  augmentationsByTaskRef: ReadonlyMap<string, TaskBoardTaskAugmentation>;
  columnRoles: TaskBoardColumnRoles;
  moving: boolean;
  movingTaskId: string | null;
  actionMenuTaskId: string | null;
  actionMenuPosition: TaskBoardMenuPosition | null;
  configuringColumns: boolean;
  dropPreviewColumn: string | null;
  openRoleMenuColumn: string | null;
  onRoleMenuColumnChange: (columnSlug: string | null) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onDragStart: (taskId: string, status: string) => void;
  onDragOverStatus: (event: DragEvent, status: string) => void;
  onDragEnd: () => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onLoadMoreColumn: (columnSlug: string) => void;
  onColumnScroll: (event: Event, columnSlug: string) => void;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
  onHideColumn: (columnSlug: string) => void;
  onMoveColumnLeft: (columnSlug: string) => void;
  onMoveColumnRight: (columnSlug: string) => void;
}

export const ROLE_LABELS: Record<TaskBoardColumnRole, string> = {
  pickup: 'Pickup',
  progress: 'In progress',
  review: 'Review',
  done: 'Done'
};

export const ROLE_ORDER: TaskBoardColumnRole[] = ['pickup', 'progress', 'review', 'done'];

export const ROLE_HELP: Record<TaskBoardColumnRole, string> = {
  pickup: 'Ready work. Agents and humans can pick tickets up from this column.',
  progress: 'Active work. Moving tickets here marks them as being worked on.',
  review:
    'Implementation and automated review are complete. Linked pull requests can now be merged or closed.',
  done: 'All pull requests linked to the ticket are merged or closed.'
};

export const columnRoleActive = (
  role: TaskBoardColumnRole,
  columnSlug: string,
  columnRoles: TaskBoardColumnRoles
): boolean => {
  if (role === 'pickup') return columnRoles.pickupColumn === columnSlug;
  if (role === 'progress') return columnRoles.progressColumn === columnSlug;
  if (role === 'review') return columnRoles.reviewColumn === columnSlug;
  return columnRoles.doneColumn === columnSlug;
};

export const optionToNullableColumn = (columnSlug: string): string | null =>
  columnSlug === '' ? null : columnSlug;

export const formatCreatedAt = (createdAt: string): string =>
  new Date(createdAt).toLocaleDateString();

export const formatTaskDisplayId = (task: TaskBoardTask): string => {
  if (task.number && task.projectSlug) {
    return `${task.projectSlug.toUpperCase()}-${task.number}`;
  }

  if (task.number) return `#${task.number}`;

  return task.id.slice(0, 8);
};

export const formatPullRequestLabel = (prUrl: string): string => {
  try {
    const url = new URL(prUrl);
    const [owner, repository, resource, number] = url.pathname.split('/').filter(Boolean);

    if (owner && repository && resource === 'pull' && number) {
      return `${owner}/${repository} #${number}`;
    }
  } catch {
    return prUrl;
  }

  return prUrl;
};

export const buildTaskBoardMenuStyle = (
  menuPosition: TaskBoardMenuPosition | null
): TaskBoardMenuStyle =>
  menuPosition
    ? {
        left: `${menuPosition.x}px`,
        top: `${menuPosition.y}px`
      }
    : undefined;

export const buildTaskBoardMoveActions = (
  task: TaskBoardTask,
  statusOptions: SelectOption[],
  onMoveTask: (taskId: string, status: string) => void
): TaskBoardMoveAction[] =>
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

export const buildTaskBoardColumns = ({
  boardColumns,
  statusOptions,
  visibleTaskCounts,
  augmentationsByTaskRef,
  columnRoles,
  moving,
  movingTaskId,
  actionMenuTaskId,
  actionMenuPosition,
  configuringColumns,
  dropPreviewColumn,
  openRoleMenuColumn,
  onRoleMenuColumnChange,
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
  onHideColumn,
  onMoveColumnLeft,
  onMoveColumnRight
}: BuildTaskBoardColumnsInput): TaskBoardColumnData[] =>
  boardColumns.map((column, index): TaskBoardColumnData => {
    const visibleCount = visibleTaskCounts[column.slug] ?? 20;
    const visibleTasks = column.tasks.slice(0, visibleCount);
    const activeRoles = ROLE_ORDER.filter((role) =>
      columnRoleActive(role, column.slug, columnRoles)
    );
    const dropPreviewActive = dropPreviewColumn === column.slug;

    return {
      column,
      visibleTasks: visibleTasks.map((task) => {
        const augmentation = augmentationsByTaskRef.get(task.id) ?? null;

        return {
          augmentation,
          pullRequests: (augmentation?.prUrls ?? []).map((url) => ({
            label: formatPullRequestLabel(url),
            url
          })),
          task,
          displayId: formatTaskDisplayId(task),
          createdAtLabel: formatCreatedAt(task.createdAt),
          moving: moving && movingTaskId === task.id,
          menuOpen: actionMenuTaskId === task.id,
          menuStyle:
            actionMenuTaskId === task.id ? buildTaskBoardMenuStyle(actionMenuPosition) : undefined,
          moveActions: buildTaskBoardMoveActions(task, statusOptions, onMoveTask),
          onClick: () => onOpenTask(task),
          onPointerDown: (event) => {
            const target = event.target;
            event.currentTarget.draggable =
              !(target instanceof Element) ||
              target.closest('[data-task-card-interactive]') === null;
          },
          onPrLinkClick: (event) => {
            event.stopPropagation();
          },
          onOpenMenu: (event) => onOpenTaskMenu(event as unknown as MouseEvent, task.id),
          onDragStart: () => onDragStart(task.id, task.status),
          onDragEnd,
          onKeyDown: (event) => {
            if (event.key !== 'Enter' && event.key !== ' ') return;

            const target = event.target;
            if (
              target instanceof Element &&
              target.closest('[data-task-card-interactive]') !== null
            ) {
              return;
            }

            event.preventDefault();
            onOpenTask(task);
          },
          onStopMenuClick: (event) => {
            event.stopPropagation();
          }
        };
      }),
      taskCount: column.tasks.length,
      activeRoles: activeRoles.map((role) => ({ role })),
      hasMoreTasks: visibleTasks.length < column.tasks.length,
      dropPreviewActive,
      roleMenu: {
        buttonLabel: `Column role settings for ${column.name}`,
        menuLabel: `Column roles for ${column.name}`,
        open: openRoleMenuColumn === column.slug,
        disabled: configuringColumns,
        onToggle: (event) => {
          event.preventDefault();
          event.stopPropagation();
          onRoleMenuColumnChange(openRoleMenuColumn === column.slug ? null : column.slug);
        },
        onStopClick: (event) => {
          event.stopPropagation();
        },
        items: ROLE_ORDER.map((role) => {
          const active = activeRoles.includes(role);
          const disabled = configuringColumns || active;

          return {
            role,
            label: `Set ${ROLE_LABELS[role]}`,
            help: ROLE_HELP[role],
            active,
            disabled,
            onClick: (event) => {
              event.preventDefault();
              event.stopPropagation();
              onSetColumnRole(column.slug, role);
              onRoleMenuColumnChange(null);
            }
          };
        })
      },
      viewControls: {
        canMoveLeft: index > 0,
        canMoveRight: index < boardColumns.length - 1,
        onHide: () => onHideColumn(column.slug),
        onMoveLeft: () => onMoveColumnLeft(column.slug),
        onMoveRight: () => onMoveColumnRight(column.slug)
      },
      onDragOver: (event) => onDragOverStatus(event as unknown as DragEvent, column.slug),
      onDrop: (event) => onDropOnStatus(event as unknown as DragEvent, column.slug),
      onScroll: (event) => onColumnScroll(event, column.slug),
      onLoadMore: () => onLoadMoreColumn(column.slug)
    };
  });
