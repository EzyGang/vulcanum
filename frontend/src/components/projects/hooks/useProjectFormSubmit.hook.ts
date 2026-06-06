import type { Signal } from '@preact/signals';
import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { createProject, updateProject } from '../../../services/projects/projects.service';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';

interface UseProjectFormSubmitOptions {
  projectId: string | null;
  enabled: Signal<boolean>;
  pickupColumn: Signal<string>;
  progressColumn: Signal<string>;
  targetColumn: Signal<string>;
  promptTemplate: Signal<string>;
  repoUrl: Signal<string>;
  agentsMd: Signal<string>;
  opencodeConfig: Signal<string>;
  providerId: Signal<string>;
  externalProjectId: Signal<string>;
}

export const useProjectFormSubmit = (options: UseProjectFormSubmitOptions) => {
  const {
    projectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    opencodeConfig,
    providerId,
    externalProjectId
  } = options;

  const [, setLocation] = useLocation();
  const formError = useSignal<string | null>(null);
  const submitting = useSignal(false);

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProject>[0]) => createProject(input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProject>[1] }) =>
      updateProject(id, input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
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
              pickupColumn: pickupColumn.value || undefined,
              progressColumn: progressColumn.value || undefined,
              targetColumn: targetColumn.value || undefined,
              promptTemplate: promptTemplate.value || undefined,
              repoUrl: repoUrl.value || undefined,
              agentsMd: agentsMd.value || undefined,
              opencodeConfig: opencodeConfig.value || undefined,
              providerId: providerId.value || undefined
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
            providerId: providerId.value,
            enabled: enabled.value,
            pickupColumn: pickupColumn.value || undefined,
            progressColumn: progressColumn.value || undefined,
            targetColumn: targetColumn.value || undefined,
            promptTemplate: promptTemplate.value,
            repoUrl: repoUrl.value || undefined,
            agentsMd: agentsMd.value || undefined,
            opencodeConfig: opencodeConfig.value || undefined
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
