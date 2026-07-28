import type {
  GenerateCodeResponse,
  RenameWorkerRequest,
  UpdateWorkerStatusRequest,
  Worker
} from '../../types/workers';
import { del, get, patch, post } from '../../utils/api/request';

export const listWorkers = (): Promise<Worker[]> => get<Worker[]>('/workers');

export const generateCode = (): Promise<GenerateCodeResponse> =>
  post<GenerateCodeResponse>('/workers/codes');

export const updateWorkerStatus = (id: string, data: UpdateWorkerStatusRequest): Promise<Worker> =>
  patch<Worker>(`/workers/${id}/status`, data);

export const renameWorker = (id: string, data: RenameWorkerRequest): Promise<Worker> =>
  patch<Worker>(`/workers/${id}/name`, data);

export const deleteWorker = (id: string): Promise<void> => del<void>(`/workers/${id}`);
