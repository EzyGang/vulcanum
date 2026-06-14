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
import type { ModelProviderConfig } from '../../../types/modelProviders';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

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
  const credentials = useSignal<Record<string, string>>({});
  const formError = useSignal<string | null>(null);
  const formSubmitting = useSignal(false);

  const selectedCatalogProvider = catalog?.providers.find((p) => p.id === providerKey.value);

  const resetForm = useCallback(() => {
    showForm.value = false;
    editId.value = null;
    providerKey.value = '';
    displayName.value = '';
    credentials.value = {};
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
    credentials.value = provider.credentials ?? {};
    formError.value = null;
    showForm.value = true;
  }, []);

  const handleProviderChange = useCallback((value: string) => {
    providerKey.value = value;
    credentials.value = {};
  }, []);

  const handleCredentialChange = useCallback((key: string, value: string) => {
    credentials.value = { ...credentials.value, [key]: value };
  }, []);

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
        if (editId.value) {
          await updateMutation.mutateAsync({
            id: editId.value,
            input: {
              displayName: displayName.value || undefined,
              credentials: credentials.value
            }
          });
        } else {
          await createMutation.mutateAsync({
            providerKey: providerKey.value,
            displayName: displayName.value || undefined,
            credentials: credentials.value
          });
        }
        resetForm();
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save model provider';
      } finally {
        formSubmitting.value = false;
      }
    },
    [createMutation, updateMutation, resetForm]
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
      credentials,
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
