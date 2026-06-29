import type {
  CreateTaskRequest,
  CreateTaskResponse,
  MoveTaskRequest,
  MoveTaskResponse,
  TaskBoardResponse,
  TaskLabelResponse,
  TaskProviderProject,
  UpdateTaskRequest,
  UpdateTaskResponse
} from '../../types/task-board';
import { del, get, patch, post, put } from '../../utils/api/request';

export const listTaskBoardProjects = (): Promise<TaskProviderProject[]> =>
  get<TaskProviderProject[]>('/task-board/projects');

export const getTaskBoard = (
  providerId: string,
  externalProjectId: string
): Promise<TaskBoardResponse> =>
  get<TaskBoardResponse>(
    `/task-board/providers/${providerId}/projects/${encodeURIComponent(externalProjectId)}`
  );

export const createTask = (
  providerId: string,
  externalProjectId: string,
  input: CreateTaskRequest
): Promise<CreateTaskResponse> =>
  post<CreateTaskResponse>(
    `/task-board/providers/${providerId}/projects/${encodeURIComponent(externalProjectId)}/tasks`,
    input
  );

export const updateTask = (
  providerId: string,
  taskId: string,
  input: UpdateTaskRequest
): Promise<UpdateTaskResponse> =>
  patch<UpdateTaskResponse>(
    `/task-board/providers/${providerId}/tasks/${encodeURIComponent(taskId)}`,
    input
  );

export const moveTask = (providerId: string, input: MoveTaskRequest): Promise<MoveTaskResponse> =>
  patch<MoveTaskResponse>(
    `/task-board/providers/${providerId}/tasks/${encodeURIComponent(input.taskId)}/status`,
    {
      status: input.status
    }
  );

export const addTaskLabel = (
  providerId: string,
  taskId: string,
  labelId: string
): Promise<TaskLabelResponse> =>
  put<TaskLabelResponse>(
    `/task-board/providers/${providerId}/tasks/${encodeURIComponent(taskId)}/labels/${encodeURIComponent(labelId)}`,
    {}
  );

export const removeTaskLabel = (
  providerId: string,
  taskId: string,
  labelId: string
): Promise<TaskLabelResponse> =>
  del<TaskLabelResponse>(
    `/task-board/providers/${providerId}/tasks/${encodeURIComponent(taskId)}/labels/${encodeURIComponent(labelId)}`
  );
