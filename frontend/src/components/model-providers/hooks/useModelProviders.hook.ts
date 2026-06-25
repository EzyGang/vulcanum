import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import {
  createModelProvider,
  deleteModelProvider,
  getModelProviderCatalog,
  listModelProviders,
  updateModelProvider
} from '../../../services/model-providers/model-providers.service';
import type { ModelProviderConfig } from '../../../types/model-providers';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { formatDateTime } from '../../../utils/format';
import { useOpenAiDeviceAuth } from './useOpenAiDeviceAuth.hook';

type AuthMethod = 'api_key' | 'device_oauth';

interface SaveButtonLabelInput {
  formSubmitting: boolean;
  deviceFlowPending: boolean;
  deviceOauth: boolean;
  editing: boolean;
}

const saveButtonLabel = ({
  formSubmitting,
  deviceFlowPending,
  deviceOauth,
  editing
}: SaveButtonLabelInput): string => {
  if (formSubmitting) {
    return deviceFlowPending ? 'Waiting...' : 'Saving...';
  }
  if (deviceOauth) {
    return 'Connect ChatGPT';
  }
  return editing ? 'Update' : 'Create';
};

const deviceFlowApprovalMessage = (nextPollAt: string | null): string => {
  if (!nextPollAt) {
    return 'Waiting for approval in your browser.';
  }
  return `Waiting for approval. Next poll: ${formatDateTime(nextPollAt)}`;
};

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
  const {
    data: { deviceFlow, deviceFlowStatus, nextPollAt },
    actions: { resetDeviceFlow, startOpenAiDeviceAuth }
  } = useOpenAiDeviceAuth();

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
    resetDeviceFlow();
    formError.value = null;
    formSubmitting.value = false;
  }, [resetDeviceFlow]);

  const handleShowCreate = useCallback(() => {
    resetForm();
    showForm.value = true;
  }, [resetForm]);

  const handleShowEdit = useCallback(
    (provider: ModelProviderConfig) => {
      editId.value = provider.id;
      providerKey.value = provider.providerKey;
      displayName.value = provider.displayName;
      authMethod.value = provider.authType === 'device_oauth' ? 'device_oauth' : 'api_key';
      credentials.value = {};
      resetDeviceFlow();
      formError.value = null;
      showForm.value = true;
    },
    [resetDeviceFlow]
  );

  const handleProviderChange = useCallback(
    (value: string) => {
      providerKey.value = value;
      authMethod.value = 'api_key';
      credentials.value = {};
      resetDeviceFlow();
    },
    [resetDeviceFlow]
  );

  const handleAuthMethodChange = useCallback(
    (value: AuthMethod) => {
      authMethod.value = value;
      credentials.value = {};
      resetDeviceFlow();
    },
    [resetDeviceFlow]
  );

  const handleCredentialChange = useCallback((key: string, value: string) => {
    credentials.value = { ...credentials.value, [key]: value };
  }, []);

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
          await startOpenAiDeviceAuth({
            displayName: displayName.value || undefined,
            onConnected: async () => {
              await invalidate('model-providers');
              resetForm();
            },
            onPollingError: (message: string) => {
              formError.value = message;
              formSubmitting.value = false;
            }
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
    [createMutation, updateMutation, startOpenAiDeviceAuth, deviceFlowStatus, resetForm]
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
      deviceFlowApprovalMessage: deviceFlowApprovalMessage(nextPollAt.value),
      formError,
      formSubmitting,
      saveButtonLabel: saveButtonLabel({
        formSubmitting: formSubmitting.value,
        deviceFlowPending: deviceFlowStatus.value === 'pending',
        deviceOauth: authMethod.value === 'device_oauth',
        editing: !!editId.value
      }),
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
