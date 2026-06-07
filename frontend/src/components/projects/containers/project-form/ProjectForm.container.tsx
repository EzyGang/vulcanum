import type { JSX } from 'preact';
import { ProjectFormFieldsProvider } from '../../context/ProjectFormFieldsContext';
import { ProjectFormLookupProvider } from '../../context/ProjectFormLookupContext';
import { ProjectFormMetaProvider } from '../../context/ProjectFormMetaContext';
import { ProjectFormProviderContextProvider } from '../../context/ProjectFormProviderContext';
import { useProjectForm } from '../../hooks/project-form/useProjectForm.hook';
import { ProjectFormView } from '../../ui/project-form/ProjectForm.view';

export const ProjectFormContainer = ({ projectId }: { projectId: string | null }): JSX.Element => {
  const { meta, provider, lookup, fields } = useProjectForm(projectId);

  return (
    <ProjectFormMetaProvider value={meta}>
      <ProjectFormProviderContextProvider value={provider}>
        <ProjectFormLookupProvider value={lookup}>
          <ProjectFormFieldsProvider value={fields}>
            <ProjectFormView />
          </ProjectFormFieldsProvider>
        </ProjectFormLookupProvider>
      </ProjectFormProviderContextProvider>
    </ProjectFormMetaProvider>
  );
};
