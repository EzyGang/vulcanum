import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useDeleteConfirm } from '../../../hooks/useDeleteConfirm.hook';
import {
  cancelChatGptAuth,
  createModelProvider,
  deleteModelProvider,
  getChatGptAuthStatus,
  getModelProviderCatalog,
  listModelProviders,
  startChatGptAuth,
  updateModelProvider
} from '../../../services/model-providers/model-providers.service';
import type { ChatGptAuthStartResponse, ModelProviderConfig } from '../../../types/modelProviders';
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
  const authType = useSignal<'api_key' | 'chatgpt_oauth'>('api_key');
  const displayName = useSignal('');
  const credentials = useSignal<Record<string, string>>({});
  const formError = useSignal<string | null>(null);
  const formSubmitting = useSignal(false);
  const chatGptAttempt = useSignal<ChatGptAuthStartResponse | null>(null);

  const resetForm = useCallback(() => {
    showForm.value = false;
    editId.value = null;
    providerKey.value = '';
    authType.value = 'api_key';
    displayName.value = '';
    credentials.value = {};
    chatGptAttempt.value = null;
    formError.value = null;
    formSubmitting.value = false;
  }, []);

  const selectedCatalogProvider = catalog?.providers.find((p) => p.id === providerKey.value);
  const chatGptAuthQuery = useApiQuery(
    ['chatgpt-auth', chatGptAttempt.value?.attemptId ?? ''],
    () => getChatGptAuthStatus(chatGptAttempt.value?.attemptId ?? ''),
    {
      enabled: !!chatGptAttempt.value,
      refetchInterval:
        chatGptAttempt.value && chatGptAuthQueryStatusIsLive(chatGptAttempt.value)
          ? chatGptAttempt.value.pollIntervalSeconds * 1000
          : false
    }
  );

  const startChatGptMutation = useApiMutation(
    (input: Parameters<typeof startChatGptAuth>[0]) => startChatGptAuth(input),
    {
      onSuccess: (attempt) => {
        chatGptAttempt.value = attempt;
      }
    }
  );

  const cancelChatGptMutation = useApiMutation((attemptId: string) => cancelChatGptAuth(attemptId));

  useEffect(() => {
    const status = chatGptAuthQuery.data?.status;
    if (status === 'complete') {
      invalidate('model-providers');
      resetForm();
    }
    if (status === 'expired' || status === 'failed') {
      formError.value = chatGptAuthQuery.data?.error ?? 'ChatGPT login failed';
    }
  }, [chatGptAuthQuery.data?.status]);

  const handleShowCreate = useCallback(() => {
    resetForm();
    showForm.value = true;
  }, [resetForm]);

  const handleShowEdit = useCallback((provider: ModelProviderConfig) => {
    editId.value = provider.id;
    providerKey.value = provider.providerKey;
    authType.value = provider.authType;
    displayName.value = provider.displayName;
    credentials.value = provider.credentials ?? {};
    formError.value = null;
    showForm.value = true;
  }, []);

  const handleProviderChange = useCallback((value: string) => {
    providerKey.value = value;
    credentials.value = {};
    chatGptAttempt.value = null;
    if (value !== 'openai') {
      authType.value = 'api_key';
    }
  }, []);

  const handleAuthTypeChange = useCallback((value: string) => {
    authType.value = value === 'chatgpt_oauth' ? 'chatgpt_oauth' : 'api_key';
    credentials.value = {};
    chatGptAttempt.value = null;
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

      if (!editId.value && authType.value === 'chatgpt_oauth') {
        formSubmitting.value = true;
        try {
          await startChatGptMutation.mutateAsync({ displayName: displayName.value || undefined });
        } catch (err) {
          formError.value = err instanceof Error ? err.message : 'Failed to start ChatGPT login';
        } finally {
          formSubmitting.value = false;
        }
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
            authType: authType.value,
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
    [createMutation, updateMutation, startChatGptMutation, resetForm]
  );

  const handleCancelChatGptAuth = useCallback(async () => {
    if (!chatGptAttempt.value) {
      return;
    }
    await cancelChatGptMutation.mutateAsync(chatGptAttempt.value.attemptId);
    chatGptAttempt.value = null;
  }, [cancelChatGptMutation]);

  return {
    data: {
      catalogProviders: catalog?.providers ?? [],
      providers: providers ?? [],
      selectedCatalogProvider,
      showForm,
      editId,
      providerKey,
      authType,
      displayName,
      credentials,
      chatGptAttempt,
      chatGptAuthStatus: chatGptAuthQuery.data,
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
      onAuthTypeChange: handleAuthTypeChange,
      onDisplayNameChange: (value: string) => {
        displayName.value = value;
      },
      onCredentialChange: handleCredentialChange,
      onCancelChatGptAuth: handleCancelChatGptAuth,
      onSave: handleSave,
      onConfirmDelete: handleConfirmDelete,
      onCancelDelete: handleCancelDelete,
      onDelete: handleDelete
    }
  };
};

const chatGptAuthQueryStatusIsLive = (attempt: ChatGptAuthStartResponse): boolean =>
  new Date(attempt.expiresAt).getTime() > Date.now();
