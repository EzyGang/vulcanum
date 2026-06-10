export type WorkRunStatus =
  | 'pending'
  | 'dispatched'
  | 'running'
  | 'completed'
  | 'failed'
  | 'stalled';

export const CANCELLABLE_STATUSES: WorkRunStatus[] = ['running', 'dispatched'];

export interface WorkRunListItem {
  id: string;
  externalTaskRef: string;
  projectConfigId: string;
  workerId: string | null;
  workerName: string | null;
  status: WorkRunStatus;
  promptText: string;
  repoUrl: string;
  taskTitle: string | null;
  taskSlug: string | null;
  resultPrUrl: string | null;
  resultExitCode: number | null;
  tokensUsed: number | null;
  inputTokens?: number | null;
  outputTokens?: number | null;
  cacheReadTokens?: number | null;
  cacheWriteTokens?: number | null;
  modelUsed?: string | null;
  durationMs: number | null;
  createdAt: string;
}
