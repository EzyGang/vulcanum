import type { SelectOption } from '../../types/shared';
import type { TaskBoard, TaskBoardTask } from '../../types/task-board';

export type TaskBoardColumnRole = 'pickup' | 'progress' | 'review' | 'done';

export type TaskBoardHelpCard = 'proxy' | 'roles' | 'automation';

export interface TaskBoardColumnRoles {
  pickupColumn: string;
  progressColumn: string;
  targetColumn: string;
  reviewPickupColumn: string | null;
}

export interface TaskBoardSettingsFormState {
  promptTemplate: string;
  agentsMd: string;
  reviewEnabled: string;
  reviewPickupColumn: string;
  reviewMaxTurns: string;
  reviewPromptTemplate: string;
  maxInProgressTasks: string;
}

export interface TaskBoardViewData {
  selectedProjectKey: string | null;
  board?: TaskBoard;
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
}

export interface TaskBoardActions {
  onTitleInput: (event: Event) => void;
  onBodyInput: (event: Event) => void;
  onStatusChange: (status: string) => void;
  onSubmitTask: (event: Event) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onToggleRepo: (repoFullName: string) => void;
  onSettingsPromptInput: (event: Event) => void;
  onSettingsAgentsInput: (event: Event) => void;
  onSettingsMaxInProgressInput: (event: Event) => void;
  onSettingsReviewEnabledChange: (value: string) => void;
  onSettingsReviewPickupColumnChange: (value: string) => void;
  onSettingsReviewMaxTurnsInput: (event: Event) => void;
  onSettingsReviewPromptInput: (event: Event) => void;
  onSetColumnRole: (columnSlug: string | null, role: TaskBoardColumnRole) => void;
  onSubmitSettings: (event: Event) => void;
  onToggleAutomation: () => void;
  onDismissHelpCard: (card: TaskBoardHelpCard) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onCloseTask: () => void;
  onDragStart: (taskId: string, status: string) => void;
  onDragOverStatus: (event: DragEvent, status: string) => void;
  onDragEnd: () => void;
  onDropOnStatus: (event: DragEvent, status: string) => void;
  onOpenCreateTask: () => void;
  onCloseCreateTask: () => void;
  onOpenSettings: () => void;
  onCloseSettings: () => void;
  onOpenTaskMenu: (event: MouseEvent, taskId: string) => void;
  onCloseTaskMenu: () => void;
  onLoadMoreColumn: (columnSlug: string) => void;
  onColumnScroll: (event: Event, columnSlug: string) => void;
}

export interface TaskBoardViewProps {
  data: TaskBoardViewData;
  form: TaskBoardFormState;
  status: TaskBoardStatusState;
  actions: TaskBoardActions;
}
