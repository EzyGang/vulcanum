import type { JSX } from 'preact';
import { useLocation } from 'wouter-preact';
import { useProjectForm } from '../hooks/useProjectForm.hook';
import { ProjectFormView } from '../ui/ProjectForm.view';

interface ProjectFormContainerProps {
  projectId: string | null;
}

export const ProjectFormContainer = ({ projectId }: ProjectFormContainerProps): JSX.Element => {
  const [_, setLocation] = useLocation();
  const form = useProjectForm(projectId);

  return (
    <ProjectFormView
      data={{
        isEdit: form.isEdit,
        providers: form.providers,
        providerId: form.providerId,
        kaneoProjectId: form.kaneoProjectId,
        enabled: form.enabled,
        pickupColumn: form.pickupColumn,
        progressColumn: form.progressColumn,
        targetColumn: form.targetColumn,
        promptTemplate: form.promptTemplate,
        repoUrl: form.repoUrl,
        agentsMd: form.agentsMd,
        columns: form.columns,
        columnsLoading: form.columnsLoading,
        lookupProjectName: form.lookupProjectName,
        lookupError: form.lookupError,
        lookedUp: form.lookedUp,
        showProviderForm: form.showProviderForm,
        newProviderName: form.newProviderName,
        newProviderUrl: form.newProviderUrl,
        newProviderKey: form.newProviderKey,
        newProviderType: form.newProviderType,
        providerFormError: form.providerFormError,
        providerSubmitting: form.providerSubmitting
      }}
      status={{
        submitting: form.submitting,
        formError: form.formError,
        projectLoading: form.projectLoading
      }}
      actions={{
        onLookup: form.handleLookup,
        onSubmit: form.handleSubmit,
        onCancel: () => setLocation('/projects'),
        onCreateProvider: form.handleCreateProvider,
        onShowProviderForm: form.onShowProviderForm,
        onCancelProviderForm: form.onCancelProviderForm,
        onProviderChange: form.onProviderChange,
        onProjectIdChange: form.onProjectIdChange,
        onEnabledChange: form.onEnabledChange,
        onPromptTemplateChange: form.onPromptTemplateChange,
        onRepoUrlChange: form.onRepoUrlChange,
        onAgentsMdChange: form.onAgentsMdChange
      }}
    />
  );
};
