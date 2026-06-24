import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import {
  createModelProvider,
  deleteModelProvider,
  getModelProviderCatalog,
  listModelProviders,
  pollModelProviderDeviceFlow,
  startModelProviderDeviceFlow,
  updateModelProvider
} from '../../../services/model-providers/model-providers.service';
import type { ModelProviderConfig, StartDeviceFlowResponse } from '../../../types/model-providers';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

type AuthMethod = 'api_key' | 'device_oauth';

const sleep = (milliseconds: number): Promise<void> =>
  new Promise((resolve) => window.setTimeout(resolve, milliseconds));

export const useModelProviders = () => {
  const { data: catalog, isLoading: catalogLoading } = useApiQuery(['model-provider-catalog'], () =>
    getModelProviderCatalog()
  );
  const {
    data: providers,
    isLoading: loading,
    error
  } = useApiQuery(['model-providers'], () => listModelProviders());

  const createMutation = useApiMutation(
    (input: Parameters<typeof createModelProvider>[0]) => createModelProvider(input),
    { onSuccess: () => invalidate('model-providers') }
  );
  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateModelProvider>[1] }) =>
      updateModelProvider(id, input),
    { onSuccess: () => invalidate('model-providers') }
  );
  const deleteMutation = useApiMutation((id: string) => deleteModelProvider(id), {
    onSuccess: () => invalidate('model-providers')
  });
  const startDeviceFlowMutation = useApiMutation(
    (input: Parameters<typeof startModelProviderDeviceFlow>[0]) =>
      startModelProviderDeviceFlow(input)
  );
  const pollDeviceFlowMutation = useApiMutation((attemptId: string) =>
    pollModelProviderDeviceFlow(attemptId)
  );

  const {
    deletingId: deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  } = useDeleteConfirm('model provider', deleteMutation);

  const showForm = useSignal(false);
  const editId = useSignal<string | null>(null);
  const providerKey = useSignal('');
  const displayName = useSignal('');
  const authMethod = useSignal<AuthMethod>('api_key');
  const credentials = useSignal<Record<string, string>>({});
  const deviceFlow = useSignal<StartDeviceFlowResponse | null>(null);
  const deviceFlowStatus = useSignal<'idle' | 'pending' | 'connected'>('idle');
  const nextPollAt = useSignal<string | null>(null);
  const formError = useSignal<string | null>(null);
  const formSubmitting = useSignal(false);

  const selectedCatalogProvider = catalog?.providers.find((p) => p.id === providerKey.value);

  const resetForm = useCallback(() => {
    showForm.value = false;
    editId.value = null;
    providerKey.value = '';
    displayName.value = '';
    authMethod.value = 'api_key';
    credentials.value = {};
    deviceFlow.value = null;
    deviceFlowStatus.value = 'idle';
    nextPollAt.value = null;
    formError.value = null;
    formSubmitting.value = false;
  }, []);

  const handleShowCreate = useCallback(() => {
    resetForm();
    showForm.value = true;
  }, [resetForm]);

  const handleShowEdit = useCallback((provider: ModelProviderConfig) => {
    editId.value = provider.id;
    providerKey.value = provider.providerKey;
    displayName.value = provider.displayName;
    authMethod.value = provider.authType === 'device_oauth' ? 'device_oauth' : 'api_key';
    credentials.value = {};
    deviceFlow.value = null;
    deviceFlowStatus.value = 'idle';
    nextPollAt.value = null;
    formError.value = null;
    showForm.value = true;
  }, []);

  const handleProviderChange = useCallback((value: string) => {
    providerKey.value = value;
    authMethod.value = 'api_key';
    credentials.value = {};
    deviceFlow.value = null;
    deviceFlowStatus.value = 'idle';
    nextPollAt.value = null;
  }, []);

  const handleAuthMethodChange = useCallback((value: AuthMethod) => {
    authMethod.value = value;
    credentials.value = {};
    deviceFlow.value = null;
    deviceFlowStatus.value = 'idle';
    nextPollAt.value = null;
  }, []);

  const handleCredentialChange = useCallback((key: string, value: string) => {
    credentials.value = { ...credentials.value, [key]: value };
  }, []);

  const pollUntilConnected = useCallback(
    async (attemptId: string, intervalSeconds: number) => {
      let delayMs = intervalSeconds * 1000;
      while (deviceFlow.value?.attemptId === attemptId) {
        await sleep(delayMs);
        if (deviceFlow.value?.attemptId !== attemptId) return;
        const response = await pollDeviceFlowMutation.mutateAsync(attemptId);
        if (response.status === 'connected') {
          deviceFlowStatus.value = 'connected';
          await invalidate('model-providers');
          resetForm();
          return;
        }
        nextPollAt.value = response.nextPollAt;
        delayMs = Math.max(new Date(response.nextPollAt).getTime() - Date.now(), 1000);
      }
    },
    [pollDeviceFlowMutation, resetForm]
  );

  const nonEmptyCredentials = (): Record<string, string> =>
    Object.fromEntries(
      Object.entries(credentials.value).filter(([, value]) => value.trim() !== '')
    );

  const handleSave = useCallback(
    async (e: Event) => {
      e.preventDefault();
      formError.value = null;
      if (!providerKey.value) {
        formError.value = 'Provider is required';
        return;
      }

      formSubmitting.value = true;
      try {
        if (providerKey.value === 'openai' && authMethod.value === 'device_oauth') {
          const flow = await startDeviceFlowMutation.mutateAsync({
            providerKey: 'openai',
            deviceProvider: 'openai_chatgpt',
            displayName: displayName.value || undefined
          });
          deviceFlow.value = flow;
          deviceFlowStatus.value = 'pending';
          nextPollAt.value = null;
          pollUntilConnected(flow.attemptId, flow.intervalSeconds).catch((err) => {
            formError.value = err instanceof Error ? err.message : 'Failed to poll device flow';
            deviceFlowStatus.value = 'idle';
            formSubmitting.value = false;
          });
          return;
        }

        const sanitizedCredentials = nonEmptyCredentials();
        if (!editId.value && Object.keys(sanitizedCredentials).length === 0) {
          formError.value = 'At least one credential is required';
          return;
        }

        if (editId.value) {
          const hasCredentials = Object.keys(sanitizedCredentials).length > 0;
          await updateMutation.mutateAsync({
            id: editId.value,
            input: {
              displayName: displayName.value || undefined,
              authType: hasCredentials ? 'api_key' : undefined,
              credentials: hasCredentials ? sanitizedCredentials : undefined
            }
          });
        } else {
          await createMutation.mutateAsync({
            providerKey: providerKey.value,
            displayName: displayName.value || undefined,
            authType: 'api_key',
            credentials: sanitizedCredentials
          });
        }
        resetForm();
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save model provider';
      } finally {
        if (deviceFlowStatus.value !== 'pending') {
          formSubmitting.value = false;
        }
      }
    },
    [createMutation, updateMutation, startDeviceFlowMutation, pollUntilConnected, resetForm]
  );

  return {
    data: {
      catalogProviders: catalog?.providers ?? [],
      providers: providers ?? [],
      selectedCatalogProvider,
      showForm,
      editId,
      providerKey,
      displayName,
      authMethod,
      credentials,
      deviceFlow,
      deviceFlowStatus,
      nextPollAt,
      formError,
      formSubmitting,
      deleteConfirmId,
      deleteError
    },
    status: { loading, catalogLoading, error },
    actions: {
      onShowCreate: handleShowCreate,
      onShowEdit: handleShowEdit,
      onCancelForm: resetForm,
      onProviderChange: handleProviderChange,
      onAuthMethodChange: handleAuthMethodChange,
      onDisplayNameChange: (value: string) => {
        displayName.value = value;
      },
      onCredentialChange: handleCredentialChange,
      onSave: handleSave,
      onConfirmDelete: handleConfirmDelete,
      onCancelDelete: handleCancelDelete,
      onDelete: handleDelete
    }
  };
};
