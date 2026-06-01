import type { JSX } from 'preact';
import type { WorkRunStatus } from '../../../types/runs';
import { useRunEvents } from '../hooks/useRunEvents.hook';
import { RunEventTimeline } from '../ui/RunEventTimeline.view';

const CANCELLABLE_STATUSES: WorkRunStatus[] = ['running', 'dispatched'];

interface RunEventTimelineContainerProps {
  runId: string;
  status: WorkRunStatus;
}

export const RunEventTimelineContainer = ({
  runId,
  status
}: RunEventTimelineContainerProps): JSX.Element => {
  const isLive = status === 'running' || status === 'dispatched';
  const canCancel = CANCELLABLE_STATUSES.includes(status);

  const { data, status: rs, actions } = useRunEvents({ runId, isLive });

  return (
    <RunEventTimeline
      isLive={isLive}
      canCancel={canCancel}
      events={data.events}
      hasMore={data.hasMore}
      loading={rs.loading}
      error={rs.error}
      cancelling={rs.cancelling}
      cancelError={rs.cancelError}
      onCancel={actions.handleCancel}
    />
  );
};
