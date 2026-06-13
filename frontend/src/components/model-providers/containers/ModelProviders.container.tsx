import type { JSX } from 'preact';
import { useModelProviders } from '../hooks/useModelProviders.hook';
import { ModelProvidersView } from '../ui/ModelProviders.view';

export const ModelProvidersContainer = (): JSX.Element => {
  const { data, status, actions } = useModelProviders();
  return <ModelProvidersView data={data} status={status} actions={actions} />;
};
