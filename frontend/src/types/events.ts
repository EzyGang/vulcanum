export interface RunEvent {
  sequence: number;
  eventType: string;
  payload: Record<string, unknown>;
}

export interface RunEventsResponse {
  events: RunEvent[];
  hasMore: boolean;
}
