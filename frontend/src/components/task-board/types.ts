import type { JSX } from 'preact';
import type { SelectOption } from '../../types/shared';
import type {
  TaskBoard,
  TaskBoardLabel,
  TaskBoardTask,
  TaskBoardTaskAugmentation
} from '../../types/task-board';

export type TaskBoardColumnRole = 'pickup' | 'progress' | 'review' | 'done';

export type TaskBoardHelpCard = 'proxy' | 'roles' | 'automation' | 'lifecycle-labels';

export interface TaskBoardColumnRoles {
  pickupColumn: string;
  progressColumn: string;
  reviewColumn: string;
  doneColumn: string;
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

export interface TaskBoardMenuPosition {
  x: number;
  y: number;
}

export type TaskBoardMenuStyle = Pick<JSX.CSSProperties, 'left' | 'top'> | undefined;

export interface TaskBoardTaskCardData {
  task: TaskBoardTask;
  augmentation: TaskBoardTaskAugmentation | null;
  displayId: string;
  createdAtLabel: string;
  moving: boolean;
  menuOpen: boolean;
  menuStyle: TaskBoardMenuStyle;
  moveActions: TaskBoardMoveAction[];
  onClick: () => void;
  onOpenMenu: JSX.MouseEventHandler<HTMLButtonElement>;
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

export interface TaskBoardColumnViewControls {
  canMoveLeft: boolean;
  canMoveRight: boolean;
  onHide: () => void;
  onMoveLeft: () => void;
  onMoveRight: () => void;
}

export interface TaskBoardColumnData {
  column: TaskBoard['columns'][number];
  visibleTasks: TaskBoardTaskCardData[];
  taskCount: number;
  activeRoles: TaskBoardRoleBadgeData[];
  hasMoreTasks: boolean;
  dropPreviewActive: boolean;
  roleMenu: TaskBoardRoleMenuData;
  viewControls: TaskBoardColumnViewControls;
  onDragOver: JSX.DragEventHandler<HTMLElement>;
  onDrop: JSX.DragEventHandler<HTMLElement>;
  onScroll: JSX.UIEventHandler<HTMLDivElement>;
  onLoadMore: () => void;
}

export interface TaskBoardHiddenColumnData {
  column: TaskBoard['columns'][number];
  onShow: () => void;
}

export interface TaskBoardColumnPreferences {
  hiddenColumnSlugs: string[];
  columnOrder: string[];
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
  hiddenColumns: TaskBoardHiddenColumnData[];
  hasCustomColumnView: boolean;
  helpCards: TaskBoardHelpCardItem[];
  automationLabel: string;
  statusOptions: SelectOption[];
  repoItems: SelectOption[];
  selectedRepoNames: string[];
  selectedTask: TaskBoardTask | null;
  selectedTaskAugmentation: TaskBoardTaskAugmentation | null;
  availableLabels: TaskBoardLabel[];
  createDialogOpen: boolean;
  settingsDialogOpen: boolean;
  actionMenuTaskId: string | null;
  actionMenuPosition: TaskBoardMenuPosition | null;
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
  editTitle: string;
  editBody: string;
  editLabelIds: string[];
  editError: string | null;
  settings: TaskBoardSettingsFormState;
}

export interface TaskBoardStatusState {
  loading: boolean;
  error: string | null;
  creating: boolean;
  movingTaskId: string | null;
  moving: boolean;
  updatingTask: boolean;
  updatingTaskLabel: boolean;
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
  onEditTaskTitleInput: (event: Event) => void;
  onEditTaskBodyInput: (event: Event) => void;
  onSubmitTaskEdit: (event: Event) => void;
  onToggleTaskLabel: (labelId: string, checked: boolean) => void;
  onDeleteLabel: (labelId: string) => void;
  onToggleRepo: (repoFullName: string) => void;
  onFilterRepos: (event: Event) => void;
  onSettingsPromptInput: (event: Event) => void;
  onResetSettingsPrompt: () => void;
  onSettingsAgentsInput: (event: Event) => void;
  onSettingsMaxInProgressInput: (event: Event) => void;
  onSettingsReviewEnabledChange: (value: string) => void;
  onSettingsReviewMaxTurnsInput: (event: Event) => void;
  onSettingsReviewPromptInput: (event: Event) => void;
  onResetSettingsReviewPrompt: () => void;
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
  onReviewColumnChange: (value: string) => void;
  onDoneColumnChange: (value: string) => void;
  onShowColumn: (columnSlug: string) => void;
  onHideColumn: (columnSlug: string) => void;
  onMoveColumnLeft: (columnSlug: string) => void;
  onMoveColumnRight: (columnSlug: string) => void;
  onResetColumnView: () => void;
}

export interface TaskBoardViewProps {
  data: TaskBoardViewData;
  form: TaskBoardFormState;
  status: TaskBoardStatusState;
  actions: TaskBoardActions;
}
