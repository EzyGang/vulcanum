import type { JSX } from 'preact';
import { useLocation } from 'wouter-preact';
import { useProjectForm } from '../hooks/useProjectForm.hook';
import { ProjectFormView } from '../ui/ProjectForm.view';

interface ProjectFormContainerProps {
  projectId: string | null;
}

export const ProjectFormContainer = ({ projectId }: ProjectFormContainerProps): JSX.Element => {
  const [_, setLocation] = useLocation();
  const {
    isEdit,
    projectLoading,
    kaneoProjectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    submitting,
    formError,
    columns,
    columnsLoading,
    columnsFetched,
    columnKaneoId: _columnKaneoId,
    handleKaneoIdChange,
    handleSubmit
  } = useProjectForm(projectId);

  return (
    <ProjectFormView
      data={{
        isEdit,
        kaneoProjectId,
        enabled,
        pickupColumn,
        progressColumn,
        targetColumn,
        promptTemplate,
        repoUrl,
        agentsMd,
        columns,
        columnsLoading,
        columnsFetched
      }}
      status={{ submitting, formError, projectLoading }}
      actions={{
        onKaneoIdChange: handleKaneoIdChange,
        onSubmit: handleSubmit,
        onCancel: () => setLocation('/projects')
      }}
    />
  );
};
