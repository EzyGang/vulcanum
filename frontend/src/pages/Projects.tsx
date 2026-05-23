import type { JSX } from 'preact';
import { ProjectsContainer } from '../components/projects/containers/Projects.container';

export const Projects = (): JSX.Element => (
  <div class='flex flex-col flex-1 px-6 py-8 max-w-5xl w-full mx-auto gap-6'>
    <ProjectsContainer />
  </div>
);
