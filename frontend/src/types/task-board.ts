import type { WorkRunStatus, WorkRunType } from './runs';

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

export interface TaskBoardRelatedWorkRun {
  id: string;
  status: WorkRunStatus;
  workType: WorkRunType;
  tokensUsed: number | null;
  inputTokens?: number | null;
  outputTokens?: number | null;
  cacheReadTokens?: number | null;
  cacheWriteTokens?: number | null;
  modelUsed?: string | null;
  createdAt: string;
}

export interface TaskBoardTaskRelatedRuns {
  externalTaskRef: string;
  runs: TaskBoardRelatedWorkRun[];
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
  relatedTaskRuns: TaskBoardTaskRelatedRuns[];
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
