import type { WorkRunListItem, WorkRunStatus } from '../../types/runs';
import { get } from '../../utils/api/request';

interface ListRunsParams {
  status?: WorkRunStatus;
  limit?: number;
  offset?: number;
}

export const listRuns = (params?: ListRunsParams): Promise<WorkRunListItem[]> => {
  const query: Record<string, string> = {};
  if (params?.status) {
    query.status = params.status;
  }
  if (params?.limit !== undefined) {
    query.limit = String(params.limit);
  }
  if (params?.offset !== undefined) {
    query.offset = String(params.offset);
  }
  return get<WorkRunListItem[]>('/runs', query);
};
