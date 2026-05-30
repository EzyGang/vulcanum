import type { JSX } from 'preact';
import { ProvidersContainer } from '../components/providers/containers/Providers.container';

export const Providers = (): JSX.Element => (
  <div class='flex flex-col flex-1 px-6 py-8 max-w-5xl w-full mx-auto gap-6'>
    <ProvidersContainer />
  </div>
);
