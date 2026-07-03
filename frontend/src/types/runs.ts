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
  pull_request_review: 'PR'
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
  reviewTargetPrUrl: string | null;
  reviewTargetRepoFullName: string | null;
  resultPrUrl: string | null;
  resultExitCode: number | null;
  tokensUsed: number | null;
  inputTokens?: number | null;
  outputTokens?: number | null;
  cacheReadTokens?: number | null;
  cacheWriteTokens?: number | null;
  modelUsed?: string | null;
  finishStatus?: string | null;
  resultSummary?: string | null;
  finishBlockedReason?: string | null;
  finishNextColumn?: string | null;
  durationMs: number | null;
  createdAt: string;
}
