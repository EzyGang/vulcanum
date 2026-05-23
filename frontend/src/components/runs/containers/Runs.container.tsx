import type { JSX } from 'preact';
import { useRuns } from '../hooks/useRuns.hook';
import { RunsView } from '../ui/Runs.view';

export const RunsContainer = (): JSX.Element => {
  const { data, status, actions } = useRuns();

  return <RunsView data={data} status={status} actions={actions} />;
};
