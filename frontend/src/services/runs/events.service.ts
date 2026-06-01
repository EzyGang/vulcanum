import type { RunEvent, RunEventsResponse } from '../../types/events';
import { get, post } from '../../utils/api/request';

interface ListRunEventsParams {
  afterSequence?: number;
  limit?: number;
}

export const listRunEvents = (
  runId: string,
  params?: ListRunEventsParams
): Promise<RunEventsResponse> => {
  const query: Record<string, number> = {};
  if (params?.afterSequence !== undefined) {
    query.after_sequence = params.afterSequence;
  }
  if (params?.limit !== undefined) {
    query.limit = params.limit;
  }
  return get<RunEventsResponse>(`/jobs/${runId}/events`, query);
};

export const cancelRun = (runId: string): Promise<void> => post<void>(`/runs/${runId}/cancel`);

export const listRecentRunEvents = (runId: string): Promise<RunEvent[]> =>
  get<RunEvent[]>(`/runs/${runId}/events/recent`);
