export interface TaskProviderProject {
  providerId: string;
  providerType: 'kaneo';
  workspaceId: string;
  externalProjectId: string;
  name: string;
  slug: string;
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
}

export interface TaskBoardResponse {
  providerId: string;
  providerType: 'kaneo';
  board: TaskBoard;
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

export interface MoveTaskRequest {
  taskId: string;
  status: string;
}

export interface MoveTaskResponse {
  taskId: string;
  status: string;
}
