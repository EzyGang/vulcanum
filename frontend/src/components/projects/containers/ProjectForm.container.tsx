import type { JSX } from 'preact';
import { useProjectForm } from '../hooks/useProjectForm.hook';
import { ProjectFormView } from '../ui/ProjectForm.view';

interface ProjectFormContainerProps {
  projectId: string | null;
}

export const ProjectFormContainer = ({ projectId }: ProjectFormContainerProps): JSX.Element => {
  const form = useProjectForm(projectId);

  return (
    <ProjectFormView
      data={{
        isEdit: form.isEdit,
        providers: form.providers,
        providerId: form.providerId,
        externalProjectId: form.externalProjectId,
        enabled: form.enabled,
        pickupColumn: form.pickupColumn,
        progressColumn: form.progressColumn,
        targetColumn: form.targetColumn,
        promptTemplate: form.promptTemplate,
        repoUrl: form.repoUrl,
        agentsMd: form.agentsMd,
        opencodeConfig: form.opencodeConfig,
        repos: form.repos,
        reposLoading: form.reposLoading,
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
        onCancel: form.cancel,
        onCreateProvider: form.handleCreateProvider,
        onShowProviderForm: form.onShowProviderForm,
        onCancelProviderForm: form.onCancelProviderForm,
        onProviderChange: form.onProviderChange,
        onProjectIdChange: form.onProjectIdChange,
        onEnabledChange: form.onEnabledChange,
        onPromptTemplateChange: form.onPromptTemplateChange,
        onRepoUrlChange: form.onRepoUrlChange,
        onAgentsMdChange: form.onAgentsMdChange,
        onOpencodeConfigChange: form.onOpencodeConfigChange,
        onPickupColumnChange: form.onPickupColumnChange,
        onProgressColumnChange: form.onProgressColumnChange,
        onTargetColumnChange: form.onTargetColumnChange
      }}
    />
  );
};
