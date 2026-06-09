import type { JSX } from 'preact';
import { ProjectFormContainer } from '../components/projects/containers/project-form/ProjectForm.container';
import { PageLayout } from '../components/shared/ui/PageLayout.view';

interface ProjectsPageProps {
  projectId?: string;
}

export const ProjectsFormPage = ({ projectId }: ProjectsPageProps): JSX.Element => (
  <PageLayout maxWidth='3xl'>
    <ProjectFormContainer projectId={projectId ?? null} />
  </PageLayout>
);
