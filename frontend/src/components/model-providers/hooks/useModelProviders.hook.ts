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
import type { SelectOption } from '../../../types/shared';
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
  const isChatGptAuth = authType.value === 'chatgpt_oauth';
  const submitLabel = submitButtonLabel(
    formSubmitting.value,
    isChatGptAuth,
    !!editId.value,
    !!chatGptAttempt.value
  );
  const authTypeItems: SelectOption[] = [
    { value: 'api_key', label: 'OpenAI API Key' },
    { value: 'chatgpt_oauth', label: 'ChatGPT Pro/Plus' }
  ];
  const chatGptAuthQuery = useApiQuery(
    ['chatgpt-auth', chatGptAttempt.value?.attemptId ?? ''],
    () => getChatGptAuthStatus(chatGptAttempt.value?.attemptId ?? ''),
    {
      enabled: !!chatGptAttempt.value,
      refetchInterval: (query) => {
        const status = query.state.data?.status;
        if (
          !chatGptAttempt.value ||
          chatGptAuthStatusIsTerminal(status) ||
          !chatGptAuthQueryStatusIsLive(chatGptAttempt.value)
        ) {
          return false;
        }
        return chatGptAttempt.value.pollIntervalSeconds * 1000;
      }
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
    const pollIntervalSeconds = chatGptAuthQuery.data?.pollIntervalSeconds;
    if (
      status === 'pending' &&
      chatGptAttempt.value &&
      pollIntervalSeconds &&
      chatGptAttempt.value.pollIntervalSeconds !== pollIntervalSeconds
    ) {
      chatGptAttempt.value = { ...chatGptAttempt.value, pollIntervalSeconds };
    }
    if (status === 'expired' || status === 'failed') {
      formError.value = chatGptAuthQuery.data?.error ?? 'ChatGPT login failed';
      chatGptAttempt.value = null;
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

  const handleDisplayNameInput = useCallback((e: Event) => {
    displayName.value = (e.target as HTMLInputElement).value;
  }, []);

  const providerRows = (providers ?? []).map((provider) => ({
    provider,
    name: provider.displayName || provider.providerKey,
    providerKey: provider.providerKey,
    authLabel: provider.authType === 'chatgpt_oauth' ? 'ChatGPT Pro/Plus' : 'API Key',
    credentialMetadata:
      provider.authType === 'chatgpt_oauth'
        ? provider.oauthMetadata?.email || provider.oauthMetadata?.accountId || 'Connected'
        : Object.keys(provider.credentials ?? {}).join(', ') || '—',
    onEdit: () => handleShowEdit(provider)
  }));
  const credentialFields = (selectedCatalogProvider?.env ?? []).map((envName) => ({
    name: envName,
    value: credentials.value[envName] ?? '',
    onInput: (e: Event) => handleCredentialChange(envName, (e.target as HTMLInputElement).value)
  }));

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
          const input =
            authType.value === 'chatgpt_oauth'
              ? { displayName: displayName.value || undefined }
              : {
                  displayName: displayName.value || undefined,
                  credentials: credentials.value
                };
          await updateMutation.mutateAsync({
            id: editId.value,
            input
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
      catalogProviderItems: (catalog?.providers ?? []).map((provider) => ({
        value: provider.id,
        label: provider.name
      })),
      providerRows,
      credentialFields,
      authTypeItems,
      isChatGptAuth,
      showAuthTypeSelect: providerKey.value === 'openai',
      showCredentialFields: !!selectedCatalogProvider && !isChatGptAuth,
      submitLabel,
      submitDisabled: formSubmitting.value || !!chatGptAttempt.value,
      showForm,
      editId,
      providerKey,
      authType,
      displayName,
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
      onCancelForm: resetForm,
      onProviderChange: handleProviderChange,
      onAuthTypeChange: handleAuthTypeChange,
      onDisplayNameInput: handleDisplayNameInput,
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

const chatGptAuthStatusIsTerminal = (status?: string): boolean =>
  status === 'complete' || status === 'failed' || status === 'expired';

const submitButtonLabel = (
  submitting: boolean,
  isChatGptAuth: boolean,
  isEdit: boolean,
  hasChatGptAttempt: boolean
): string => {
  if (submitting) {
    return 'Saving...';
  }
  if (isChatGptAuth && !isEdit) {
    return hasChatGptAttempt ? 'Waiting for Login' : 'Start Device Login';
  }
  return isEdit ? 'Update' : 'Create';
};
