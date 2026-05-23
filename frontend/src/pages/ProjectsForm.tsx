import type { JSX } from 'preact';
import { ProjectFormContainer } from '../components/projects/containers/ProjectForm.container';

interface ProjectsPageProps {
  projectId?: string;
}

export const ProjectsFormPage = ({ projectId }: ProjectsPageProps): JSX.Element => (
  <div class='flex flex-col flex-1 px-6 py-8 max-w-3xl w-full mx-auto gap-6'>
    <ProjectFormContainer projectId={projectId ?? null} />
  </div>
);
