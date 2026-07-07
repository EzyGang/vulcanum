import type { WorkRunTokenUsage } from './runs';

export interface TaskProviderProject {
  providerId: string;
  providerType: string;
  workspaceId: string;
  externalProjectId: string;
  name: string;
  slug: string;
}

export interface TaskBoardLabel {
  id: string;
  name: string;
  color: string;
}

export interface TaskBoardTaskAugmentation extends WorkRunTokenUsage {
  externalTaskRef: string;
  tokensUsed: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  finishedRunsCount: number;
  updatedAt: string;
}

export interface TaskBoardTask {
  id: string;
  title: string;
  projectId: string;
  description?: string | null;
  status: string;
  priority: string;
  number?: number | null;
  projectSlug?: string | null;
  assigneeName?: string | null;
  createdAt: string;
  updatedAt?: string | null;
  labels: TaskBoardLabel[];
}

export interface TaskBoardColumn {
  id: string;
  name: string;
  slug: string;
  isFinal?: boolean | null;
  tasks: TaskBoardTask[];
}

export interface TaskBoardProject {
  id: string;
  name: string;
  slug: string;
}

export interface TaskBoard {
  project: TaskBoardProject;
  columns: TaskBoardColumn[];
  labels: TaskBoardLabel[];
}

export interface TaskBoardResponse {
  providerId: string;
  providerType: string;
  board: TaskBoard;
  taskAugmentations: TaskBoardTaskAugmentation[];
}

export interface CreateTaskRequest {
  title: string;
  body: string;
  status?: string;
  priority?: string;
}

export interface CreateTaskResponse {
  task: TaskBoardTask;
}

export interface UpdateTaskRequest {
  title: string;
  body: string;
}

export interface UpdateTaskResponse {
  task: TaskBoardTask;
}

export interface MoveTaskRequest {
  taskId: string;
  status: string;
}

export interface MoveTaskResponse {
  taskId: string;
  status: string;
}

export interface TaskLabelResponse {
  taskId: string;
  labelId: string;
}

export interface TaskLabelDeleteResponse {
  labelId: string;
}
