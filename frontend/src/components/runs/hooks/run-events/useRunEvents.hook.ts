import { useCallback, useMemo } from 'preact/hooks';
import { cancelRun, listRecentRunEvents } from '../../../../services/runs/events.service';
import { invalidate } from '../../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../../utils/api/query/hooks';

const POLL_INTERVAL_MS = 5_000;

interface UseRunEventsParams {
  runId: string;
  isLive: boolean;
}

export const useRunEvents = ({ runId, isLive }: UseRunEventsParams) => {
  const queryKey = useMemo(() => ['run-events', runId] as const, [runId]);

  const { data, isLoading, error } = useApiQuery(queryKey, () => listRecentRunEvents(runId), {
    refetchInterval: isLive ? POLL_INTERVAL_MS : false
  });

  const cancelMutation = useApiMutation<void, void>(() => cancelRun(runId), {
    onSuccess: () => invalidate('runs')
  });

  const handleCancel = useCallback(() => {
    cancelMutation.mutate();
  }, [cancelMutation]);

  return {
    data: { events: data ?? [], hasMore: false },
    status: {
      loading: isLoading,
      error,
      cancelling: cancelMutation.isPending,
      cancelError: cancelMutation.error
    },
    actions: { handleCancel }
  };
};
