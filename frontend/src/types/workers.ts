export type WorkerStatus = 'idle' | 'busy' | 'disconnected' | 'unhealthy';

export interface Worker {
  id: string;
  name: string;
  status: WorkerStatus;
  lastSeen: string | null;
  capabilities: Record<string, unknown>;
  createdAt: string;
  activeJobs: number;
  maxConcurrentJobs: number;
  consecutiveErrors: number;
}

export interface GenerateCodeResponse {
  code: string;
  expiresAt: string;
}

export interface UpdateWorkerStatusRequest {
  status: 'idle' | 'unhealthy';
}
