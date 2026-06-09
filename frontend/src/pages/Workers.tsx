import type { JSX } from 'preact';
import { WorkersContainer } from '../components/workers/containers/Workers.container';

export const Workers = (): JSX.Element => (
  <div class='flex flex-col flex-1 px-4 sm:px-6 py-8 max-w-5xl w-full mx-auto gap-6 animate-fade-in'>
    <WorkersContainer />
  </div>
);
