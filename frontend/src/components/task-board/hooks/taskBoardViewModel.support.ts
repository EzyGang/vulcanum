import type { SelectOption } from '../../../types/shared';
import type { TaskBoard, TaskBoardTask } from '../../../types/task-board';
import type {
  TaskBoardColumnData,
  TaskBoardColumnRole,
  TaskBoardColumnRoles,
  TaskBoardHelpCard,
  TaskBoardMoveAction,
  TaskBoardProjectSettingsData,
  TaskBoardRepositorySettingsData,
  TaskBoardReviewSettingsData,
  TaskBoardRoleSelectData,
  TaskBoardSettingsFormState
} from '../types';

export interface UseTaskBoardViewModelInput {
  board?: TaskBoard;
  statusOptions: SelectOption[];
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  selectedTask: TaskBoardTask | null;
  visibleTaskCounts: Record<string, number>;
  columnRoles: TaskBoardColumnRoles;
  moving: boolean;
  movingTaskId: string | null;
  actionMenuTaskId: string | null;
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
      hasOverrides: boolean;
      roleSelects: TaskBoardRoleSelectData[];
    };
    projectSettings: TaskBoardProjectSettingsData;
    reviewSettings: TaskBoardReviewSettingsData;
    selectedTaskCreatedAtLabel: string | null;
    selectedTaskMoveActions: TaskBoardMoveAction[];
  };
  actions: {
    onFilterRepos: (event: Event) => void;
    onPickupColumnChange: (value: string) => void;
    onProgressColumnChange: (value: string) => void;
    onDoneColumnChange: (value: string) => void;
    onReviewColumnChange: (value: string) => void;
  };
}

export const ROLE_LABELS: Record<TaskBoardColumnRole, string> = {
  pickup: 'Pickup',
  progress: 'In progress',
  done: 'Done',
  review: 'Review'
};

export const ROLE_ORDER: TaskBoardColumnRole[] = ['pickup', 'progress', 'done', 'review'];

export const ROLE_HELP: Record<TaskBoardColumnRole, string> = {
  pickup: 'Ready work. Agents and humans can pick tickets up from this column.',
  progress: 'Active work. Moving tickets here marks them as being worked on.',
  done: 'Completed work. Moving tickets here closes the board workflow.',
  review: 'Review pickup. Automated review work starts from this column when enabled.'
};

export const columnRoleActive = (
  role: TaskBoardColumnRole,
  columnSlug: string,
  columnRoles: TaskBoardColumnRoles
): boolean => {
  if (role === 'pickup') return columnRoles.pickupColumn === columnSlug;
  if (role === 'progress') return columnRoles.progressColumn === columnSlug;
  if (role === 'done') return columnRoles.targetColumn === columnSlug;
  return columnRoles.reviewPickupColumn === columnSlug;
};

export const optionToNullableColumn = (columnSlug: string): string | null =>
  columnSlug === '' ? null : columnSlug;

export const formatCreatedAt = (createdAt: string): string =>
  new Date(createdAt).toLocaleDateString();
