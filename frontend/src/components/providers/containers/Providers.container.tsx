import type { JSX } from 'preact';
import { useProviders } from '../hooks/useProviders.hook';
import { ProvidersView } from '../ui/Providers.view';

export const ProvidersContainer = (): JSX.Element => {
  const {
    providers,
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
  } = useProviders();

  return (
    <ProvidersView
      data={{
        providers,
        deleteConfirmId,
        deleteError,
        showForm,
        editId,
        name: name.value,
        url: url.value,
        apiKey: apiKey.value,
        providerType: providerType.value,
        formError: formError.value,
        formSubmitting: formSubmitting.value
      }}
      status={{ loading, error }}
      actions={{
        onShowCreate: handleShowCreate,
        onShowEdit: handleShowEdit,
        onCancelForm: handleCancelForm,
        onSave: handleSave,
        onConfirmDelete: handleConfirmDelete,
        onCancelDelete: handleCancelDelete,
        onDelete: handleDelete,
        onNameChange,
        onUrlChange,
        onApiKeyChange,
        onProviderTypeChange
      }}
    />
  );
};
