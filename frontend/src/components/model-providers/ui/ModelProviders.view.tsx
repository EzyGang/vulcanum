import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import type {
  ChatGptAuthStartResponse,
  ChatGptAuthStatusResponse,
  ModelProviderConfig
} from '../../../types/modelProviders';
import type { SelectOption } from '../../../types/shared';
import type { ApiError } from '../../../utils/api/client';
import { Button } from '../../shared/ui/Button.view';
import { ConfirmDelete } from '../../shared/ui/ConfirmDelete.view';
import { EmptyState } from '../../shared/ui/EmptyState.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';
import { SectionHeader } from '../../shared/ui/SectionHeader.view';
import { Select } from '../../shared/ui/Select.view';
import { Table } from '../../shared/ui/Table.view';

interface ModelProvidersViewProps {
  data: {
    catalogProviderItems: SelectOption[];
    providerRows: {
      provider: ModelProviderConfig;
      name: string;
      providerKey: string;
      authLabel: string;
      credentialMetadata: string;
    }[];
    credentialFields: string[];
    authTypeItems: SelectOption[];
    isChatGptAuth: boolean;
    showAuthTypeSelect: boolean;
    showCredentialFields: boolean;
    submitLabel: string;
    submitDisabled: boolean;
    showForm: Signal<boolean>;
    editId: Signal<string | null>;
    providerKey: Signal<string>;
    authType: Signal<'api_key' | 'chatgpt_oauth'>;
    displayName: Signal<string>;
    credentials: Signal<Record<string, string>>;
    chatGptAttempt: Signal<ChatGptAuthStartResponse | null>;
    chatGptAuthStatus?: ChatGptAuthStatusResponse;
    formError: Signal<string | null>;
    formSubmitting: Signal<boolean>;
    deleteConfirmId: Signal<string | null>;
    deleteError: Signal<string | null>;
  };
  status: {
    loading: boolean;
    catalogLoading: boolean;
    error: ApiError | null;
  };
  actions: {
    onShowCreate: () => void;
    onShowEdit: (provider: ModelProviderConfig) => void;
    onCancelForm: () => void;
    onProviderChange: (value: string) => void;
    onAuthTypeChange: (value: string) => void;
    onDisplayNameChange: (value: string) => void;
    onCredentialChange: (key: string, value: string) => void;
    onCancelChatGptAuth: () => void;
    onSave: (e: Event) => void;
    onConfirmDelete: (id: string) => void;
    onCancelDelete: () => void;
    onDelete: (id: string) => void;
  };
}

export const ModelProvidersView = ({
  data,
  status,
  actions
}: ModelProvidersViewProps): JSX.Element => (
  <div class='flex flex-col gap-4'>
    <SectionHeader
      title='Model Providers'
      hint='Connect LLM providers for app-managed agent runtime config.'
      action={
        !data.showForm.value ? (
          <Button
            variant='primary'
            class='shrink-0 whitespace-nowrap px-5'
            onClick={actions.onShowCreate}
            disabled={status.catalogLoading}
          >
            Add Model Provider
          </Button>
        ) : null
      }
    />

    {status.error && <ErrorBanner message={status.error.message} />}
    {data.deleteError.value && <ErrorBanner message={data.deleteError.value} />}
    {status.loading && <div class='text-text-muted text-sm'>Loading model providers...</div>}

    {data.showForm.value && (
      <form
        onSubmit={actions.onSave}
        class='border border-border-base bg-bg-card p-5 flex flex-col gap-4'
      >
        <div class='flex flex-col gap-2'>
          <Label for='model-provider-key'>Provider</Label>
          <Select
            id='model-provider-key'
            value={data.providerKey.value}
            onValueChange={actions.onProviderChange}
            disabled={!!data.editId.value || data.formSubmitting.value}
            placeholder='Select a provider...'
            items={data.catalogProviderItems}
          />
        </div>

        {data.showAuthTypeSelect && (
          <div class='flex flex-col gap-2'>
            <Label for='model-provider-auth-type'>Auth Method</Label>
            <Select
              id='model-provider-auth-type'
              value={data.authType.value}
              onValueChange={actions.onAuthTypeChange}
              disabled={!!data.editId.value || data.formSubmitting.value}
              items={data.authTypeItems}
            />
          </div>
        )}

        <div class='flex flex-col gap-2'>
          <Label for='model-provider-display-name'>Display Name</Label>
          <Input
            id='model-provider-display-name'
            value={data.displayName.value}
            onInput={(e) => actions.onDisplayNameChange((e.target as HTMLInputElement).value)}
            disabled={data.formSubmitting.value}
            placeholder='Production Anthropic'
          />
        </div>

        {data.showCredentialFields && (
          <div class='flex flex-col gap-3'>
            <div class='text-text-muted text-xs'>Credential fields from models.dev catalog.</div>
            {data.credentialFields.map((envName) => (
              <div class='flex flex-col gap-2' key={envName}>
                <Label for={`credential-${envName}`}>{envName}</Label>
                <Input
                  id={`credential-${envName}`}
                  type='password'
                  value={data.credentials.value[envName] ?? ''}
                  onInput={(e) =>
                    actions.onCredentialChange(envName, (e.target as HTMLInputElement).value)
                  }
                  disabled={data.formSubmitting.value}
                />
              </div>
            ))}
          </div>
        )}

        {data.isChatGptAuth && (
          <div class='border border-border-base bg-bg-input p-4 flex flex-col gap-3'>
            <div class='flex flex-col gap-1'>
              <span class='text-text-primary text-sm font-medium'>ChatGPT Pro/Plus Login</span>
              <span class='text-text-secondary text-sm'>
                This uses OpenCode-compatible OAuth. Tokens are stored encrypted and never shown
                again after connection.
              </span>
            </div>
            {data.chatGptAttempt.value && (
              <div class='flex flex-col gap-2'>
                <span class='text-text-muted text-xs uppercase tracking-wide'>
                  Verification URL
                </span>
                <a
                  class='text-text-primary text-sm underline underline-offset-4'
                  href={data.chatGptAttempt.value.verificationUri}
                  target='_blank'
                  rel='noreferrer'
                >
                  {data.chatGptAttempt.value.verificationUri}
                </a>
                <span class='text-text-muted text-xs uppercase tracking-wide'>User Code</span>
                <span class='border border-border-base bg-bg-card px-3 py-2 font-mono text-lg text-text-primary tracking-wide'>
                  {data.chatGptAttempt.value.userCode}
                </span>
                <span class='text-text-secondary text-sm'>
                  Status: {data.chatGptAuthStatus?.status ?? 'pending'}
                </span>
              </div>
            )}
          </div>
        )}

        {data.formError.value && <ErrorBanner message={data.formError.value} />}

        <div class='flex items-center gap-3'>
          <Button type='submit' variant='primary' disabled={data.submitDisabled}>
            {data.submitLabel}
          </Button>
          {data.chatGptAttempt.value && (
            <Button type='button' variant='secondary' onClick={actions.onCancelChatGptAuth}>
              Cancel Login
            </Button>
          )}
          <Button type='button' variant='secondary' onClick={actions.onCancelForm}>
            Cancel
          </Button>
        </div>
      </form>
    )}

    {!status.loading && data.providerRows.length === 0 && !data.showForm.value && (
      <EmptyState
        title='No model providers connected yet.'
        description='Connect a model provider to select app-managed runtime models for projects.'
      />
    )}

    {!status.loading && data.providerRows.length > 0 && (
      <Table>
        <Table.Head>
          <Table.HeadCell>Name</Table.HeadCell>
          <Table.HeadCell>Provider</Table.HeadCell>
          <Table.HeadCell>Auth</Table.HeadCell>
          <Table.HeadCell>Credential Metadata</Table.HeadCell>
          <Table.HeadCell>Actions</Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {data.providerRows.map((row) => (
            <Table.Row key={row.provider.id}>
              <Table.Cell>
                <span class='text-text-primary text-sm'>{row.name}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>{row.providerKey}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm'>{row.authLabel}</span>
              </Table.Cell>
              <Table.Cell>
                <span class='text-text-secondary text-sm font-mono'>{row.credentialMetadata}</span>
              </Table.Cell>
              <Table.Cell>
                <ConfirmDelete
                  itemId={row.provider.id}
                  deletingId={data.deleteConfirmId}
                  onConfirm={actions.onConfirmDelete}
                  onDelete={actions.onDelete}
                  onCancel={actions.onCancelDelete}
                  editActions={
                    <Button variant='ghost' onClick={() => actions.onShowEdit(row.provider)}>
                      Edit
                    </Button>
                  }
                />
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table>
    )}
  </div>
);
