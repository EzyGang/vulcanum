import type { JSX } from 'preact';
import type { RunEvent } from '../../../../types/events';
import type { ApiError } from '../../../../utils/api/client';
import { ErrorBanner } from '../../../shared/ui/ErrorBanner.view';
import { StatusBadge } from '../../../shared/ui/StatusBadge.view';

interface RunEventTimelineProps {
  isLive: boolean;
  events: RunEvent[];
  hasMore: boolean;
  loading: boolean;
  error: ApiError | null;
}

const summarizePayload = (payload: Record<string, unknown>): string => {
  if (payload.reason !== undefined) {
    return `reason: ${String(payload.reason)}`;
  }
  if (payload.text !== undefined) {
    return `text: ${String(payload.text).slice(0, 80)}`;
  }
  if (payload.pr_url !== undefined) {
    return `pr: ${String(payload.pr_url)}`;
  }
  if (Object.keys(payload).length === 0) {
    return '—';
  }
  return JSON.stringify(payload).slice(0, 80);
};

export const RunEventTimeline = ({
  isLive,
  events,
  hasMore,
  loading,
  error
}: RunEventTimelineProps): JSX.Element => (
  <div class='flex flex-col gap-3 bg-bg-surface border border-border-base p-4'>
    <div class='flex items-center justify-between'>
      <div class='flex items-center gap-2'>
        <span class='text-text-primary text-xs font-semibold uppercase tracking-wide'>
          Event timeline
        </span>
        {isLive && <span class='text-text-muted text-xs uppercase tracking-wider'>live</span>}
      </div>
    </div>

    {error && <ErrorBanner message={error.message} />}

    {loading && events.length === 0 && <div class='text-text-muted text-xs'>Loading events…</div>}

    {events.length === 0 && !loading ? (
      <div class='text-text-muted text-xs'>No events recorded yet.</div>
    ) : (
      <ul class='flex flex-col gap-2 max-h-64 overflow-y-auto'>
        {events.map((event) => (
          <li key={event.sequence} class='flex items-start gap-3 border-l border-border-base pl-3'>
            <span class='text-text-muted text-xs font-mono w-10 shrink-0'>#{event.sequence}</span>
            <StatusBadge status={event.eventType} />
            <span class='text-text-secondary text-xs font-mono flex-1 break-all'>
              {summarizePayload(event.payload)}
            </span>
          </li>
        ))}
      </ul>
    )}

    {hasMore && (
      <div class='text-text-muted text-xs'>Showing most recent events; older ones omitted.</div>
    )}
  </div>
);
