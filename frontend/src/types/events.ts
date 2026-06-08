export interface RunEvent {
  sequence: number;
  eventType: string;
  payload: Record<string, unknown>;
  occurredAt: string;
}

export interface RunEventsResponse {
  events: RunEvent[];
  hasMore: boolean;
}
