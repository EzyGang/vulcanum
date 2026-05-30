import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { createProvider } from '../../../services/providers/providers.service';
import { invalidate } from '../../../utils/api/query/client';

export const useProjectFormProvider = (onProviderCreated: (id: string) => void) => {
  const showProviderForm = useSignal(false);
  const newProviderName = useSignal('');
  const newProviderUrl = useSignal('');
  const newProviderKey = useSignal('');
  const providerFormError = useSignal<string | null>(null);
  const providerSubmitting = useSignal(false);

  const handleCreateProvider = useCallback(async (e: Event) => {
    e.preventDefault();
    providerFormError.value = null;

    if (!newProviderName.value || !newProviderUrl.value || !newProviderKey.value) {
      providerFormError.value = 'All fields are required';
      return;
    }

    providerSubmitting.value = true;
    try {
      const created = await createProvider({
        name: newProviderName.value,
        instanceUrl: newProviderUrl.value,
        apiKey: newProviderKey.value
      });
      invalidate('providers');
      onProviderCreated(created.id);
      showProviderForm.value = false;
      newProviderName.value = '';
      newProviderUrl.value = '';
      newProviderKey.value = '';
    } catch (err) {
      providerFormError.value = err instanceof Error ? err.message : 'Failed to create provider';
    } finally {
      providerSubmitting.value = false;
    }
  }, []);

  const onShowProviderForm = useCallback(() => {
    showProviderForm.value = true;
  }, []);

  const onCancelProviderForm = useCallback(() => {
    showProviderForm.value = false;
    providerFormError.value = null;
  }, []);

  return {
    showProviderForm,
    newProviderName,
    newProviderUrl,
    newProviderKey,
    providerFormError,
    providerSubmitting,
    handleCreateProvider,
    onShowProviderForm,
    onCancelProviderForm
  };
};
