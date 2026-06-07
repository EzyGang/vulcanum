import type { JSX } from 'preact';
import type { WorkRunStatus } from '../../../../types/runs';
import { CANCELLABLE_STATUSES } from '../../../../types/runs';
import { useRunEvents } from '../../hooks/run-events/useRunEvents.hook';
import { RunEventTimeline } from '../../ui/run-events/RunEventTimeline.view';

export interface RunEventTimelineContainerProps {
  runId: string;
  status: WorkRunStatus;
}

export const RunEventTimelineContainer = ({
  runId,
  status
}: RunEventTimelineContainerProps): JSX.Element => {
  const isLive = CANCELLABLE_STATUSES.includes(status);
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
