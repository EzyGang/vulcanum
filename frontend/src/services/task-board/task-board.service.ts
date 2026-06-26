import type {
  CreateTaskRequest,
  CreateTaskResponse,
  MoveTaskRequest,
  MoveTaskResponse,
  TaskBoardResponse,
  TaskProviderProject
} from '../../types/task-board';
import { get, patch, post } from '../../utils/api/request';

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

export const moveTask = (providerId: string, input: MoveTaskRequest): Promise<MoveTaskResponse> =>
  patch<MoveTaskResponse>(
    `/task-board/providers/${providerId}/tasks/${encodeURIComponent(input.taskId)}/status`,
    {
      status: input.status
    }
  );
