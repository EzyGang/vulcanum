import type { JSX } from 'preact';
import { ProjectFormProvider } from '../../context/ProjectFormContext';
import { useProjectForm } from '../../hooks/project-form/useProjectForm.hook';
import { ProjectFormView } from '../../ui/project-form/ProjectForm.view';

export const ProjectFormContainer = ({ projectId }: { projectId: string | null }): JSX.Element => {
  const form = useProjectForm(projectId);

  const value = {
    data: {
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
      canShowLookup: form.canShowLookup,
      canShowFields: form.canShowFields,
      showProviderForm: form.showProviderForm,
      newProviderName: form.newProviderName,
      newProviderUrl: form.newProviderUrl,
      newProviderKey: form.newProviderKey,
      newProviderType: form.newProviderType,
      providerFormError: form.providerFormError,
      providerSubmitting: form.providerSubmitting
    },
    status: {
      submitting: form.submitting,
      formError: form.formError,
      projectLoading: form.projectLoading
    },
    actions: {
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
      onTargetColumnChange: form.onTargetColumnChange,
      onNewProviderNameChange: form.onNewProviderNameChange,
      onNewProviderUrlChange: form.onNewProviderUrlChange,
      onNewProviderKeyChange: form.onNewProviderKeyChange,
      onNewProviderTypeChange: form.onNewProviderTypeChange
    }
  };

  return (
    <ProjectFormProvider value={value}>
      <ProjectFormView />
    </ProjectFormProvider>
  );
};
