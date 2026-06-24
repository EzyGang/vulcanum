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
  promptTemplateOverride: Signal<boolean>;
  repoFullNames: Signal<string[]>;
  agentsMd: Signal<string>;
  agentsMdOverride: Signal<boolean>;
  primaryModelProviderConfigId: Signal<string>;
  primaryModelProviderOverride: Signal<boolean>;
  primaryModelId: Signal<string>;
  primaryModelIdOverride: Signal<boolean>;
  smallModelProviderConfigId: Signal<string>;
  smallModelProviderOverride: Signal<boolean>;
  smallModelId: Signal<string>;
  smallModelIdOverride: Signal<boolean>;
  reviewEnabled: Signal<boolean>;
  reviewEnabledOverride: Signal<boolean>;
  reviewPickupColumn: Signal<string>;
  reviewPickupColumnOverride: Signal<boolean>;
  reviewMaxTurns: Signal<number>;
  reviewMaxTurnsOverride: Signal<boolean>;
  reviewPromptTemplate: Signal<string>;
  reviewPromptTemplateOverride: Signal<boolean>;
  maxInProgressTasks: Signal<number>;
  maxInProgressTasksOverride: Signal<boolean>;
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
    promptTemplateOverride,
    repoFullNames,
    agentsMd,
    agentsMdOverride,
    primaryModelProviderConfigId,
    primaryModelProviderOverride,
    primaryModelId,
    primaryModelIdOverride,
    smallModelProviderConfigId,
    smallModelProviderOverride,
    smallModelId,
    smallModelIdOverride,
    reviewEnabled,
    reviewEnabledOverride,
    reviewPickupColumn,
    reviewPickupColumnOverride,
    reviewMaxTurns,
    reviewMaxTurnsOverride,
    reviewPromptTemplate,
    reviewPromptTemplateOverride,
    maxInProgressTasks,
    maxInProgressTasksOverride,
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
              promptTemplate: overrideOr(
                promptTemplate,
                promptTemplateOverride,
                null,
                emptyStringAsNull
              ),
              repoFullNames: repoFullNames.value,
              agentsMd: overrideOr(agentsMd, agentsMdOverride, null, emptyStringAsNull),
              primaryModelProviderConfigId: overrideOr(
                primaryModelProviderConfigId,
                primaryModelProviderOverride,
                null,
                emptyStringAsNull
              ),
              primaryModelId: overrideOr(
                primaryModelId,
                primaryModelIdOverride,
                null,
                emptyStringAsNull
              ),
              smallModelProviderConfigId: overrideOr(
                smallModelProviderConfigId,
                smallModelProviderOverride,
                null,
                emptyStringAsNull
              ),
              smallModelId: overrideOr(smallModelId, smallModelIdOverride, null, emptyStringAsNull),
              reviewEnabled: overrideOr(reviewEnabled, reviewEnabledOverride, null),
              reviewPickupColumn: overrideOr(
                reviewPickupColumn,
                reviewPickupColumnOverride,
                null,
                emptyStringAsNull
              ),
              reviewMaxTurns: overrideOr(reviewMaxTurns, reviewMaxTurnsOverride, null),
              reviewPromptTemplate: overrideOr(
                reviewPromptTemplate,
                reviewPromptTemplateOverride,
                null,
                emptyStringAsNull
              ),
              maxInProgressTasks: overrideOr(maxInProgressTasks, maxInProgressTasksOverride, null),
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
            promptTemplate: overrideOr(
              promptTemplate,
              promptTemplateOverride,
              undefined,
              emptyStringAsUndefined
            ),
            repoFullNames: repoFullNames.value,
            agentsMd: overrideOr(agentsMd, agentsMdOverride, undefined, emptyStringAsUndefined),
            primaryModelProviderConfigId: overrideOr(
              primaryModelProviderConfigId,
              primaryModelProviderOverride,
              undefined,
              emptyStringAsUndefined
            ),
            primaryModelId: overrideOr(
              primaryModelId,
              primaryModelIdOverride,
              undefined,
              emptyStringAsUndefined
            ),
            smallModelProviderConfigId: overrideOr(
              smallModelProviderConfigId,
              smallModelProviderOverride,
              undefined,
              emptyStringAsUndefined
            ),
            smallModelId: overrideOr(
              smallModelId,
              smallModelIdOverride,
              undefined,
              emptyStringAsUndefined
            ),
            reviewEnabled: overrideOr(reviewEnabled, reviewEnabledOverride, undefined),
            reviewPickupColumn: overrideOr(
              reviewPickupColumn,
              reviewPickupColumnOverride,
              undefined,
              emptyStringAsUndefined
            ),
            reviewMaxTurns: overrideOr(reviewMaxTurns, reviewMaxTurnsOverride, undefined),
            reviewPromptTemplate: overrideOr(
              reviewPromptTemplate,
              reviewPromptTemplateOverride,
              undefined,
              emptyStringAsUndefined
            ),
            maxInProgressTasks: overrideOr(
              maxInProgressTasks,
              maxInProgressTasksOverride,
              undefined
            )
          });
        }
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save project config';
      } finally {
        submitting.value = false;
      }
    },
    [projectId, createMutation, updateMutation, setLocation]
  );

  return {
    formError,
    submitting,
    handleSubmit
  };
};

const overrideOr = <T, F>(
  field: Signal<T>,
  enabled: Signal<boolean>,
  fallback: F,
  valueForSubmit: (value: T) => T | F = (value) => value
): T | F => (enabled.value ? valueForSubmit(field.value) : fallback);

const emptyStringAsNull = (value: string): string | null => value || null;

const emptyStringAsUndefined = (value: string): string | undefined => value || undefined;
