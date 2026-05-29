import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type { IntegrationProvider } from '../../../types/projects';
import type { ApiError } from '../../../utils/api/client';
import { ProviderFormFields } from './ProviderFormFields.view';

interface ProvidersViewProps {
  data: {
    providers: IntegrationProvider[];
    deleteConfirmId: Signal<string | null>;
    deleteError: Signal<string | null>;
    showForm: Signal<boolean>;
    editId: Signal<string | null>;
    name: Signal<string>;
    url: Signal<string>;
    apiKey: Signal<string>;
    formError: Signal<string | null>;
    formSubmitting: Signal<boolean>;
  };
  status: {
    loading: boolean;
    error: ApiError | null;
  };
  actions: {
    onShowCreate: () => void;
    onShowEdit: (provider: IntegrationProvider) => void;
    onCancelForm: () => void;
    onSave: (e: Event) => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDelete: (id: string) => void;
  };
}

export const ProvidersView = ({
  data: {
    providers,
    deleteConfirmId,
    deleteError,
    showForm,
    editId,
    name,
    url,
    apiKey,
    formError,
    formSubmitting
  },
  status: { loading, error },
  actions: {
    onShowCreate,
    onShowEdit,
    onCancelForm,
    onSave,
    onConfirmDelete,
    onCancelDelete,
    onDelete
  }
}: ProvidersViewProps): JSX.Element => (
  <div class='flex flex-col gap-8'>
    <section class='flex flex-col gap-4'>
      <div class='flex items-center justify-between'>
        <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Providers</h2>
        <button
          type='button'
          onClick={onShowCreate}
          class='bg-text-primary text-bg-page text-sm font-medium uppercase tracking-wider px-4 py-3 hover:opacity-90 transition-opacity'
        >
          Add Provider
        </button>
      </div>

      {error && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {error.message}
        </div>
      )}

      {deleteError.value && (
        <div class='text-error text-sm bg-error-bg border border-error-border p-4'>
          {deleteError.value}
        </div>
      )}

      {loading && <div class='text-text-muted text-sm'>Loading providers...</div>}

      {!loading && !error && providers.length === 0 && !showForm.value && (
        <div class='flex flex-col items-center gap-4 bg-bg-card border border-border-base p-12'>
          <p class='text-text-muted text-sm'>No providers configured yet.</p>
          <p class='text-text-muted text-xs'>Add a provider to connect Kaneo projects.</p>
        </div>
      )}

      {showForm.value && (
        <ProviderFormFields
          name={name}
          url={url}
          apiKey={apiKey}
          error={formError}
          submitting={formSubmitting}
          mode={editId.value ? 'edit' : 'create'}
          onSave={onSave}
          onCancel={onCancelForm}
        />
      )}

      {!loading && providers.length > 0 && (
        <div class='overflow-x-auto'>
          <table class='w-full border-collapse'>
            <thead>
              <tr class='border-b border-border-base'>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Name
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Instance URL
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Created
                </th>
                <th class='text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3'>
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {providers.map((provider) => (
                <tr key={provider.id} class='border-b border-border-base'>
                  <td class='px-5 py-3'>
                    <span class='text-text-primary text-sm'>{provider.name}</span>
                  </td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm font-mono'>
                      {provider.instanceUrl}
                    </span>
                  </td>
                  <td class='px-5 py-3'>
                    <span class='text-text-secondary text-sm'>{provider.createdAt}</span>
                  </td>
                  <td class='px-5 py-3'>
                    {deleteConfirmId.value === provider.id ? (
                      <div class='flex items-center gap-2'>
                        <span class='text-text-muted text-xs'>Confirm?</span>
                        <button
                          type='button'
                          onClick={() => onDelete(provider.id)}
                          class='text-error text-xs uppercase tracking-wider hover:opacity-80 transition-opacity'
                        >
                          Delete
                        </button>
                        <button
                          type='button'
                          onClick={onCancelDelete}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors'
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <div class='flex items-center gap-3'>
                        <button
                          type='button'
                          onClick={() => onShowEdit(provider)}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-text-primary transition-colors'
                        >
                          Edit
                        </button>
                        <button
                          type='button'
                          onClick={() => onConfirmDelete(provider.id)}
                          class='text-text-muted text-xs uppercase tracking-wider hover:text-error transition-colors'
                        >
                          Delete
                        </button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  </div>
);
