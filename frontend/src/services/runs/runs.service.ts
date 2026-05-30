import type { WorkRunListItem, WorkRunStatus } from '../../types/runs';
import { del, get, post } from '../../utils/api/request';

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

export const deleteRun = (id: string): Promise<void> => del(`/runs/${id}`);

export const bulkDeleteRuns = (ids: string[]): Promise<{ deleted: number }> =>
  post<{ deleted: number }>('/runs/bulk-delete', { ids });

export const failRun = (id: string): Promise<WorkRunListItem> =>
  post<WorkRunListItem>(`/runs/${id}/fail`);
