import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import {
  createProvider,
  deleteProvider,
  listProviders,
  updateProvider
} from '../../../services/providers/providers.service';
import type { IntegrationProvider } from '../../../types/projects';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';
import { formatDateTime } from '../../../utils/format';

export const useProviders = () => {
  const {
    data: providers,
    isLoading: loading,
    error
  } = useApiQuery(['providers'], () => listProviders());

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProvider>[0]) => createProvider(input),
    {
      onSuccess: () => invalidate('providers')
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProvider>[1] }) =>
      updateProvider(id, input),
    {
      onSuccess: () => invalidate('providers')
    }
  );

  const deleteMutation = useApiMutation((id: string) => deleteProvider(id), {
    onSuccess: () => invalidate('providers')
  });

  const {
    deletingId: deleteConfirmId,
    deleteError,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  } = useDeleteConfirm('provider', deleteMutation);

  const formError = useSignal<string | null>(null);
  const formSubmitting = useSignal(false);

  const showForm = useSignal(false);
  const editId = useSignal<string | null>(null);
  const name = useSignal('');
  const url = useSignal('');
  const apiKey = useSignal('');
  const providerType = useSignal('kaneo');

  const resetForm = useCallback(() => {
    name.value = '';
    url.value = '';
    apiKey.value = '';
    providerType.value = 'kaneo';
    formError.value = null;
    formSubmitting.value = false;
    editId.value = null;
    showForm.value = false;
  }, []);

  const onNameChange = useCallback((value: string) => {
    name.value = value;
  }, []);
  const onUrlChange = useCallback((value: string) => {
    url.value = value;
  }, []);
  const onApiKeyChange = useCallback((value: string) => {
    apiKey.value = value;
  }, []);
  const onProviderTypeChange = useCallback((value: string) => {
    providerType.value = value;
  }, []);

  const handleShowCreate = useCallback(() => {
    resetForm();
    showForm.value = true;
  }, [resetForm]);

  const handleShowEdit = useCallback((provider: IntegrationProvider) => {
    name.value = provider.name;
    url.value = provider.instanceUrl;
    apiKey.value = provider.apiKey;
    providerType.value = provider.providerType;
    formError.value = null;
    formSubmitting.value = false;
    editId.value = provider.id;
    showForm.value = true;
  }, []);

  const handleCancelForm = useCallback(() => {
    resetForm();
  }, [resetForm]);

  const handleSave = useCallback(
    async (e: Event) => {
      e.preventDefault();
      formError.value = null;

      if (!name.value || !url.value || !apiKey.value) {
        formError.value = 'All fields are required';
        return;
      }

      formSubmitting.value = true;
      try {
        if (editId.value) {
          await updateMutation.mutateAsync({
            id: editId.value,
            input: {
              name: name.value,
              providerType: providerType.value,
              instanceUrl: url.value,
              apiKey: apiKey.value
            }
          });
        } else {
          await createMutation.mutateAsync({
            name: name.value,
            providerType: providerType.value,
            instanceUrl: url.value,
            apiKey: apiKey.value
          });
        }
        resetForm();
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save provider';
      } finally {
        formSubmitting.value = false;
      }
    },
    [editId, createMutation, updateMutation, resetForm]
  );

  return {
    providers:
      providers?.map((provider) => ({
        ...provider,
        formattedCreatedAt: formatDateTime(provider.createdAt)
      })) ?? [],
    loading,
    error,
    deleteConfirmId,
    deleteError,
    formError,
    formSubmitting,
    showForm,
    editId,
    name,
    url,
    apiKey,
    providerType,
    handleShowCreate,
    handleShowEdit,
    handleCancelForm,
    handleSave,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete,
    onNameChange,
    onUrlChange,
    onApiKeyChange,
    onProviderTypeChange
  };
};
