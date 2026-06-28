import type { JSX } from 'preact';
import type { SelectOption } from '../../types/shared';
import type { TaskBoard, TaskBoardTask } from '../../types/task-board';

export type TaskBoardColumnRole = 'pickup' | 'progress' | 'done';

export type TaskBoardHelpCard = 'proxy' | 'roles' | 'automation';

export interface TaskBoardColumnRoles {
  pickupColumn: string;
  progressColumn: string;
  targetColumn: string;
}

export interface TaskBoardSettingsFormState {
  promptTemplate: string;
  agentsMd: string;
  reviewEnabled: string;
  reviewMaxTurns: string;
  reviewPromptTemplate: string;
  maxInProgressTasks: string;
}

export interface TaskBoardHelpCardItem {
  id: TaskBoardHelpCard;
  title: string;
  body: string;
  onDismiss: () => void;
}

export interface TaskBoardMoveAction {
  value: string;
  label: string;
  onClick: JSX.MouseEventHandler<HTMLButtonElement>;
}

export interface TaskBoardTaskCardData {
  task: TaskBoardTask;
  displayId: string;
  createdAtLabel: string;
  moving: boolean;
  menuOpen: boolean;
  moveActions: TaskBoardMoveAction[];
  onClick: () => void;
  onContextMenu: JSX.MouseEventHandler<HTMLElement>;
  onDragStart: JSX.DragEventHandler<HTMLElement>;
  onDragEnd: () => void;
  onKeyDown: JSX.KeyboardEventHandler<HTMLElement>;
  onStopMenuClick: JSX.MouseEventHandler<HTMLDivElement>;
}

export interface TaskBoardRoleBadgeData {
  role: TaskBoardColumnRole;
}

export interface TaskBoardRoleMenuItem {
  role: TaskBoardColumnRole;
  label: string;
  help: string;
  active: boolean;
  disabled: boolean;
  onClick: JSX.MouseEventHandler<HTMLButtonElement>;
}

export interface TaskBoardRoleMenuData {
  buttonLabel: string;
  menuLabel: string;
  open: boolean;
  disabled: boolean;
  onToggle: JSX.MouseEventHandler<HTMLButtonElement>;
  onStopClick: JSX.MouseEventHandler<HTMLDivElement>;
  items: TaskBoardRoleMenuItem[];
}

export interface TaskBoardColumnData {
  column: TaskBoard['columns'][number];
  visibleTasks: TaskBoardTaskCardData[];
  taskCount: number;
  activeRoles: TaskBoardRoleBadgeData[];
  hasMoreTasks: boolean;
  dropPreviewActive: boolean;
  roleMenu: TaskBoardRoleMenuData;
  onDragOver: JSX.DragEventHandler<HTMLElement>;
  onDrop: JSX.DragEventHandler<HTMLElement>;
  onScroll: JSX.UIEventHandler<HTMLDivElement>;
  onLoadMore: () => void;
}

export interface TaskBoardRepositoryItem {
  value: string;
  label: string;
  checked: boolean;
  onToggle: () => void;
}

export interface TaskBoardRepositorySettingsData {
  filter: string;
  selectedRepos: TaskBoardRepositoryItem[];
  filteredRepos: TaskBoardRepositoryItem[];
  hasRepos: boolean;
  hasSelectedRepos: boolean;
  hasFilteredRepos: boolean;
  hasOverrides: boolean;
}

export interface TaskBoardRoleSelectData {
  id: string;
  label: string;
  value: string;
  options: SelectOption[];
  onValueChange: (value: string) => void;
}

export interface TaskBoardColumnSettingsData {
  hasOptions: boolean;
  hasOverrides: boolean;
  roleSelects: TaskBoardRoleSelectData[];
}

export interface TaskBoardProjectSettingsData {
  hasOverrides: boolean;
}

export interface TaskBoardReviewSettingsData {
  hasOverrides: boolean;
}

export interface TaskBoardViewData {
  selectedProjectKey: string | null;
  board?: TaskBoard;
  boardColumnCount: number;
  columns: TaskBoardColumnData[];
  helpCards: TaskBoardHelpCardItem[];
  automationLabel: string;
  statusOptions: SelectOption[];
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  selectedTask: TaskBoardTask | null;
  createDialogOpen: boolean;
  settingsDialogOpen: boolean;
  actionMenuTaskId: string | null;
  visibleTaskCounts: Record<string, number>;
  columnRoles: TaskBoardColumnRoles;
  dropPreviewColumn: string | null;
  automationEnabled: boolean;
  dismissedHelpCards: TaskBoardHelpCard[];
  repositorySettings: TaskBoardRepositorySettingsData;
  columnSettings: TaskBoardColumnSettingsData;
  projectSettings: TaskBoardProjectSettingsData;
  reviewSettings: TaskBoardReviewSettingsData;
  selectedTaskCreatedAtLabel: string | null;
  selectedTaskMoveActions: TaskBoardMoveAction[];
}

export interface TaskBoardFormState {
  title: string;
  body: string;
  status: string;
  createError: string | null;
  serverError: string | null;
  settings: TaskBoardSettingsFormState;
}

export interface TaskBoardStatusState {
  loading: boolean;
  error: string | null;
  creating: boolean;
  movingTaskId: string | null;
  moving: boolean;
  reposLoading: boolean;
  connectingRepos: boolean;
  connected: boolean;
  savingSettings: boolean;
  configuringColumns: boolean;
  savingAutomation: boolean;
  settingsDisabled: boolean;
  repoControlsDisabled: boolean;
}

export interface TaskBoardActions {
  onTitleInput: (event: Event) => void;
  onBodyInput: (event: Event) => void;
  onStatusChange: (status: string) => void;
  onSubmitTask: (event: Event) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onToggleRepo: (repoFullName: string) => void;
  onFilterRepos: (event: Event) => void;
  onSettingsPromptInput: (event: Event) => void;
  onSettingsAgentsInput: (event: Event) => void;
  onSettingsMaxInProgressInput: (event: Event) => void;
  onSettingsReviewEnabledChange: (value: string) => void;
  onSettingsReviewMaxTurnsInput: (event: Event) => void;
  onSettingsReviewPromptInput: (event: Event) => void;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
  onSubmitSettings: (event: Event) => void;
  onToggleAutomation: () => void;
  onDismissHelpCard: (card: TaskBoardHelpCard) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onCloseTask: () => void;
  onTaskDetailsOpenChange: (open: boolean) => void;
  onDragStart: (taskId: string, status: string) => void;
  onDragOverStatus: (event: DragEvent, status: string) => void;
  onDragEnd: () => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onOpenCreateTask: () => void;
  onCloseCreateTask: () => void;
  onCreateDialogOpenChange: (open: boolean) => void;
  onOpenSettings: () => void;
  onCloseSettings: () => void;
  onSettingsDialogOpenChange: (open: boolean) => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onCloseTaskMenu: () => void;
  onLoadMoreColumn: (columnSlug: string) => void;
  onColumnScroll: (event: Event, columnSlug: string) => void;
  onPickupColumnChange: (value: string) => void;
  onProgressColumnChange: (value: string) => void;
  onDoneColumnChange: (value: string) => void;
}

export interface TaskBoardViewProps {
  data: TaskBoardViewData;
  form: TaskBoardFormState;
  status: TaskBoardStatusState;
  actions: TaskBoardActions;
}
