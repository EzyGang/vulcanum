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
  repoFullNames: Signal<string[]>;
  agentsMd: Signal<string>;
  overridesOpen: Signal<boolean>;
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
    repoFullNames,
    agentsMd,
    overridesOpen,
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
              promptTemplate: overrideOrNull(promptTemplate, overridesOpen),
              repoFullNames: repoFullNames.value,
              agentsMd: overrideOrNull(agentsMd, overridesOpen),
              primaryModelProviderKey: overrideOrNull(primaryModelProviderKey, overridesOpen),
              primaryModelId: overrideOrNull(primaryModelId, overridesOpen),
              smallModelProviderKey: overrideOrNull(smallModelProviderKey, overridesOpen),
              smallModelId: overrideOrNull(smallModelId, overridesOpen),
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
            promptTemplate: overrideOrUndefined(promptTemplate, overridesOpen),
            repoFullNames: repoFullNames.value,
            agentsMd: overrideOrUndefined(agentsMd, overridesOpen),
            primaryModelProviderKey: overrideOrUndefined(primaryModelProviderKey, overridesOpen),
            primaryModelId: overrideOrUndefined(primaryModelId, overridesOpen),
            smallModelProviderKey: overrideOrUndefined(smallModelProviderKey, overridesOpen),
            smallModelId: overrideOrUndefined(smallModelId, overridesOpen)
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

const overrideOrNull = (field: Signal<string>, enabled: Signal<boolean>): string | null =>
  enabled.value ? field.value || null : null;

const overrideOrUndefined = (
  field: Signal<string>,
  enabled: Signal<boolean>
): string | undefined => (enabled.value ? field.value || undefined : undefined);
