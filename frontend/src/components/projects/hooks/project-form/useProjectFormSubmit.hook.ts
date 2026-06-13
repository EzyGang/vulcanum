import type { Signal } from '@preact/signals';
import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { createProject, updateProject } from '../../../../services/projects/projects.service';
import { invalidate } from '../../../../utils/api/query/client';
import { useApiMutation } from '../../../../utils/api/query/hooks';

interface UseProjectFormSubmitOptions {
  projectId: string | null;
  name: Signal<string>;
  enabled: Signal<boolean>;
  pickupColumn: Signal<string>;
  progressColumn: Signal<string>;
  targetColumn: Signal<string>;
  promptTemplate: Signal<string>;
  repoUrl: Signal<string>;
  agentsMd: Signal<string>;
  opencodeConfig: Signal<string>;
  primaryModelProviderKey: Signal<string>;
  primaryModelId: Signal<string>;
  smallModelProviderKey: Signal<string>;
  smallModelId: Signal<string>;
  providerId: Signal<string>;
  externalProjectId: Signal<string>;
  workspaceId: Signal<string>;
}

export const useProjectFormSubmit = (options: UseProjectFormSubmitOptions) => {
  const {
    projectId,
    name,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    opencodeConfig,
    primaryModelProviderKey,
    primaryModelId,
    smallModelProviderKey,
    smallModelId,
    providerId,
    externalProjectId,
    workspaceId
  } = options;

  const [, setLocation] = useLocation();
  const formError = useSignal<string | null>(null);
  const submitting = useSignal(false);

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProject>[0]) => createProject(input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/settings?tab=projects');
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProject>[1] }) =>
      updateProject(id, input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/settings?tab=projects');
      }
    }
  );

  const handleSubmit = useCallback(
    async (e: Event) => {
      e.preventDefault();
      formError.value = null;

      if (!promptTemplate.value) {
        formError.value = 'Prompt template is required';
        return;
      }

      submitting.value = true;

      try {
        if (projectId) {
          await updateMutation.mutateAsync({
            id: projectId,
            input: {
              enabled: enabled.value,
              // On update, undefined omits COALESCE-preserved fields while null explicitly clears nullable model fields.
              pickupColumn: pickupColumn.value || undefined,
              progressColumn: progressColumn.value || undefined,
              targetColumn: targetColumn.value || undefined,
              promptTemplate: promptTemplate.value || undefined,
              repoUrl: repoUrl.value || undefined,
              agentsMd: agentsMd.value || undefined,
              opencodeConfig: opencodeConfig.value || undefined,
              primaryModelProviderKey: primaryModelProviderKey.value || null,
              primaryModelId: primaryModelId.value || null,
              smallModelProviderKey: smallModelProviderKey.value || null,
              smallModelId: smallModelId.value || null,
              name: name.value || undefined,
              providerId: providerId.value || undefined,
              externalWorkspaceId: workspaceId.value || undefined
            }
          });
        } else {
          if (!providerId.value) {
            formError.value = 'Provider is required';
            submitting.value = false;
            return;
          }
          await createMutation.mutateAsync({
            externalProjectId: externalProjectId.value,
            externalWorkspaceId: workspaceId.value || undefined,
            name: name.value || undefined,
            providerId: providerId.value,
            enabled: enabled.value,
            pickupColumn: pickupColumn.value || undefined,
            progressColumn: progressColumn.value || undefined,
            targetColumn: targetColumn.value || undefined,
            promptTemplate: promptTemplate.value,
            repoUrl: repoUrl.value || undefined,
            agentsMd: agentsMd.value || undefined,
            opencodeConfig: opencodeConfig.value || undefined,
            primaryModelProviderKey: primaryModelProviderKey.value || undefined,
            primaryModelId: primaryModelId.value || undefined,
            smallModelProviderKey: smallModelProviderKey.value || undefined,
            smallModelId: smallModelId.value || undefined
          });
        }
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save project config';
      } finally {
        submitting.value = false;
      }
    },
    [projectId, createMutation, updateMutation]
  );

  return {
    formError,
    submitting,
    handleSubmit
  };
};
