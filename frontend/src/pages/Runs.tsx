import type { JSX } from 'preact';
import { RunsContainer } from '../components/runs/containers/Runs.container';

export const Runs = (): JSX.Element => (
  <div class='flex flex-col flex-1 px-4 sm:px-6 py-8 max-w-6xl w-full mx-auto gap-6 animate-fade-in'>
    <RunsContainer />
  </div>
);
