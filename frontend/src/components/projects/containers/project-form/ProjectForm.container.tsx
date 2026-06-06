import type { JSX } from 'preact';
import { ProjectFormProvider } from '../../context/ProjectFormContext';
import { useProjectForm } from '../../hooks/project-form/useProjectForm.hook';
import { ProjectFormView } from '../../ui/project-form/ProjectForm.view';

export const ProjectFormContainer = ({ projectId }: { projectId: string | null }): JSX.Element => (
  <ProjectFormProvider value={useProjectForm(projectId)}>
    <ProjectFormView />
  </ProjectFormProvider>
);
