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
  primaryModelProviderKey: Signal<string>;
  primaryModelProviderOverride: Signal<boolean>;
  primaryModelId: Signal<string>;
  primaryModelIdOverride: Signal<boolean>;
  smallModelProviderKey: Signal<string>;
  smallModelProviderOverride: Signal<boolean>;
  smallModelId: Signal<string>;
  smallModelIdOverride: Signal<boolean>;
  reviewEnabled: Signal<boolean>;
  reviewEnabledOverride: Signal<boolean>;
  reviewPickupColumn: Signal<string>;
  reviewPickupColumnOverride: Signal<boolean>;
  reviewMaxTurns: Signal<string>;
  reviewMaxTurnsOverride: Signal<boolean>;
  reviewPromptTemplate: Signal<string>;
  reviewPromptTemplateOverride: Signal<boolean>;
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
    primaryModelProviderKey,
    primaryModelProviderOverride,
    primaryModelId,
    primaryModelIdOverride,
    smallModelProviderKey,
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
              promptTemplate: overrideOrNull(promptTemplate, promptTemplateOverride),
              repoFullNames: repoFullNames.value,
              agentsMd: overrideOrNull(agentsMd, agentsMdOverride),
              primaryModelProviderKey: overrideOrNull(
                primaryModelProviderKey,
                primaryModelProviderOverride
              ),
              primaryModelId: overrideOrNull(primaryModelId, primaryModelIdOverride),
              smallModelProviderKey: overrideOrNull(
                smallModelProviderKey,
                smallModelProviderOverride
              ),
              smallModelId: overrideOrNull(smallModelId, smallModelIdOverride),
              reviewEnabled: overrideBoolOrNull(reviewEnabled, reviewEnabledOverride),
              reviewPickupColumn: overrideOrNull(reviewPickupColumn, reviewPickupColumnOverride),
              reviewMaxTurns: overrideNumberOrNull(reviewMaxTurns, reviewMaxTurnsOverride),
              reviewPromptTemplate: overrideOrNull(
                reviewPromptTemplate,
                reviewPromptTemplateOverride
              ),
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
            promptTemplate: overrideOrUndefined(promptTemplate, promptTemplateOverride),
            repoFullNames: repoFullNames.value,
            agentsMd: overrideOrUndefined(agentsMd, agentsMdOverride),
            primaryModelProviderKey: overrideOrUndefined(
              primaryModelProviderKey,
              primaryModelProviderOverride
            ),
            primaryModelId: overrideOrUndefined(primaryModelId, primaryModelIdOverride),
            smallModelProviderKey: overrideOrUndefined(
              smallModelProviderKey,
              smallModelProviderOverride
            ),
            smallModelId: overrideOrUndefined(smallModelId, smallModelIdOverride),
            reviewEnabled: overrideBoolOrUndefined(reviewEnabled, reviewEnabledOverride),
            reviewPickupColumn: overrideOrUndefined(reviewPickupColumn, reviewPickupColumnOverride),
            reviewMaxTurns: overrideNumberOrUndefined(reviewMaxTurns, reviewMaxTurnsOverride),
            reviewPromptTemplate: overrideOrUndefined(
              reviewPromptTemplate,
              reviewPromptTemplateOverride
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

const overrideOrNull = (field: Signal<string>, enabled: Signal<boolean>): string | null =>
  enabled.value ? field.value || null : null;

const overrideOrUndefined = (
  field: Signal<string>,
  enabled: Signal<boolean>
): string | undefined => (enabled.value ? field.value || undefined : undefined);

const overrideBoolOrNull = (field: Signal<boolean>, enabled: Signal<boolean>): boolean | null =>
  enabled.value ? field.value : null;

const overrideBoolOrUndefined = (
  field: Signal<boolean>,
  enabled: Signal<boolean>
): boolean | undefined => (enabled.value ? field.value : undefined);

const overrideNumberOrNull = (field: Signal<string>, enabled: Signal<boolean>): number | null =>
  enabled.value ? Number(field.value) || null : null;

const overrideNumberOrUndefined = (
  field: Signal<string>,
  enabled: Signal<boolean>
): number | undefined => (enabled.value ? Number(field.value) || undefined : undefined);
