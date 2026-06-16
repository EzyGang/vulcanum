export type WorkRunStatus =
  | 'pending'
  | 'dispatched'
  | 'running'
  | 'completed'
  | 'failed'
  | 'stalled';

export const CANCELLABLE_STATUSES: WorkRunStatus[] = ['running', 'dispatched'];

export type WorkRunType = 'implementation' | 'pull_request_review';

export const WORK_RUN_TYPE_LABELS: Record<WorkRunType, string> = {
  implementation: 'Implement',
  pull_request_review: 'Review'
};

export const WORK_RUN_PR_LINK_LABELS: Record<WorkRunType, string> = {
  implementation: 'PR',
  pull_request_review: 'Review PR'
};

export interface WorkRunListItem {
  id: string;
  externalTaskRef: string;
  projectConfigId: string;
  workerId: string | null;
  workerName: string | null;
  status: WorkRunStatus;
  workType: WorkRunType;
  parentWorkRunId: string | null;
  promptText: string;
  repoUrl: string;
  taskBody: string;
  taskTitle: string | null;
  taskSlug: string | null;
  reviewTargetPrUrl: string | null;
  reviewTargetRepoFullName: string | null;
  reviewUrl: string | null;
  reviewBody: string | null;
  reviewAlreadyExists: boolean;
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
