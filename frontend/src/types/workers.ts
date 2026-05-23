export type WorkerStatus = 'idle' | 'busy' | 'disconnected';

export interface Worker {
  id: string;
  name: string;
  status: WorkerStatus;
  lastSeen: string | null;
  capabilities: Record<string, unknown>;
  createdAt: string;
}

export interface GenerateCodeResponse {
  code: string;
  expiresAt: string;
}
