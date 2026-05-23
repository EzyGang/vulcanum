import type { GenerateCodeResponse, Worker } from '../../types/workers';
import { del, get, post } from '../../utils/api/request';

export const listWorkers = (): Promise<Worker[]> => get<Worker[]>('/workers');

export const generateCode = (): Promise<GenerateCodeResponse> =>
  post<GenerateCodeResponse>('/workers/codes');

export const deleteWorker = (id: string): Promise<void> => del<void>(`/workers/${id}`);
