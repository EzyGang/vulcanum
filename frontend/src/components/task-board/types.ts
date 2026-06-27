import type { SelectOption } from '../../types/shared';
import type { TaskBoard, TaskBoardTask } from '../../types/task-board';

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
}

export interface TaskBoardFormState {
  title: string;
  body: string;
  status: string;
  createError: string | null;
  serverError: string | null;
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
}

export interface TaskBoardActions {
  onTitleInput: (event: Event) => void;
  onBodyInput: (event: Event) => void;
  onStatusChange: (status: string) => void;
  onSubmitTask: (event: Event) => void;
  onMoveTask: (taskId: string, status: string) => void;
  onToggleRepo: (repoFullName: string) => void;
  onOpenTask: (task: TaskBoardTask) => void;
  onCloseTask: () => void;
  onDragStart: (taskId: string) => void;
  onDragOver: (event: DragEvent) => void;
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
