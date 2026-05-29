import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import {
  createProvider,
  deleteProvider,
  listProviders,
  updateProvider
} from '../../../services/providers/providers.service';
import type { IntegrationProvider } from '../../../types/projects';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

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

  const deleteConfirmId = useSignal<string | null>(null);
  const deleteError = useSignal<string | null>(null);
  const formError = useSignal<string | null>(null);
  const formSubmitting = useSignal(false);

  const showForm = useSignal(false);
  const editId = useSignal<string | null>(null);
  const name = useSignal('');
  const url = useSignal('');
  const apiKey = useSignal('');

  const resetForm = useCallback(() => {
    name.value = '';
    url.value = '';
    apiKey.value = '';
    formError.value = null;
    formSubmitting.value = false;
    editId.value = null;
    showForm.value = false;
  }, []);

  const handleShowCreate = useCallback(() => {
    resetForm();
    showForm.value = true;
  }, [resetForm]);

  const handleShowEdit = useCallback((provider: IntegrationProvider) => {
    name.value = provider.name;
    url.value = provider.instanceUrl;
    apiKey.value = provider.apiKey;
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
              instanceUrl: url.value,
              apiKey: apiKey.value
            }
          });
        } else {
          await createMutation.mutateAsync({
            name: name.value,
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

  const handleConfirmDelete = useCallback((id: string) => {
    deleteConfirmId.value = id;
  }, []);

  const handleCancelDelete = useCallback(() => {
    deleteConfirmId.value = null;
  }, []);

  const handleDelete = useCallback(
    async (id: string) => {
      deleteError.value = null;
      try {
        await deleteMutation.mutateAsync(id);
      } catch (err) {
        deleteError.value = err instanceof Error ? err.message : 'Failed to delete provider';
      } finally {
        deleteConfirmId.value = null;
      }
    },
    [deleteMutation]
  );

  return {
    providers: providers ?? [],
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
    handleShowCreate,
    handleShowEdit,
    handleCancelForm,
    handleSave,
    handleConfirmDelete,
    handleCancelDelete,
    handleDelete
  };
};
