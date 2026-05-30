import type { GenerateCodeResponse, UpdateWorkerStatusRequest, Worker } from '../../types/workers';
import { del, get, post, put } from '../../utils/api/request';

export const listWorkers = (): Promise<Worker[]> => get<Worker[]>('/workers');

export const generateCode = (): Promise<GenerateCodeResponse> =>
  post<GenerateCodeResponse>('/workers/codes');

export const updateWorkerStatus = (id: string, data: UpdateWorkerStatusRequest): Promise<Worker> =>
  put<Worker>(`/workers/${id}/status`, data);

export const deleteWorker = (id: string): Promise<void> => del<void>(`/workers/${id}`);
