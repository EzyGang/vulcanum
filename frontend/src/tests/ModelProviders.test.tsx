import { signal } from '@preact/signals';
import { render } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

vi.mock('../components/shared/ui/Select.view', () => ({
  Select: ({
    items = [],
    value
  }: {
    items?: { label: string; value: string }[];
    value: string;
  }) => (
    <div data-testid='select' data-value={value}>
      {items.map((item) => (
        <span key={item.value}>{item.label}</span>
      ))}
    </div>
  )
}));

import { ModelProvidersView } from '../components/model-providers/ui/ModelProviders.view';
import type { ModelProviderConfig } from '../types/modelProviders';

const catalogProviderItems = [{ value: 'openai', label: 'OpenAI' }];
const authTypeItems = [
  { value: 'api_key', label: 'OpenAI API Key' },
  { value: 'chatgpt_oauth', label: 'ChatGPT Pro/Plus' }
];

const provider: ModelProviderConfig = {
  id: 'provider-1',
  providerKey: 'openai',
  authType: 'chatgpt_oauth',
  displayName: 'OpenAI ChatGPT Pro/Plus',
  credentials: {},
  oauthMetadata: { accountId: 'acct_123' },
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z'
};

const actions = {
  onShowCreate: vi.fn(),
  onCancelForm: vi.fn(),
  onProviderChange: vi.fn(),
  onAuthTypeChange: vi.fn(),
  onDisplayNameInput: vi.fn(),
  onCancelChatGptAuth: vi.fn(),
  onSave: vi.fn(),
  onConfirmDelete: vi.fn(),
  onCancelDelete: vi.fn(),
  onDelete: vi.fn()
};

describe('ModelProvidersView', () => {
  it('shows ChatGPT device login details instead of API key fields', () => {
    const { getByText, queryByText } = render(
      <ModelProvidersView
        data={{
          catalogProviderItems,
          providerRows: [],
          credentialFields: [
            {
              name: 'OPENAI_API_KEY',
              value: '',
              onInput: vi.fn()
            }
          ],
          authTypeItems,
          isChatGptAuth: true,
          showAuthTypeSelect: true,
          showCredentialFields: false,
          submitLabel: 'Waiting for Login',
          submitDisabled: true,
          showForm: signal(true),
          editId: signal(null),
          providerKey: signal('openai'),
          authType: signal('chatgpt_oauth'),
          displayName: signal(''),
          chatGptAttempt: signal({
            attemptId: 'attempt-1',
            verificationUri: 'https://auth.openai.com/codex/device',
            userCode: 'ABCD-EFGH',
            expiresAt: '2026-01-01T00:10:00Z',
            pollIntervalSeconds: 5
          }),
          chatGptAuthStatus: { status: 'pending' },
          formError: signal(null),
          formSubmitting: signal(false),
          deleteConfirmId: signal(null),
          deleteError: signal(null)
        }}
        status={{ loading: false, catalogLoading: false, error: null }}
        actions={actions}
      />
    );

    expect(getByText('ChatGPT Pro/Plus Login')).toBeDefined();
    expect(getByText('ABCD-EFGH')).toBeDefined();
    expect(queryByText('Credential fields from models.dev catalog.')).toBeNull();
  });

  it('labels ChatGPT OAuth providers distinctly in the table', () => {
    const { getByText } = render(
      <ModelProvidersView
        data={{
          catalogProviderItems,
          providerRows: [
            {
              provider,
              name: provider.displayName,
              providerKey: provider.providerKey,
              authLabel: 'ChatGPT Pro/Plus',
              credentialMetadata: 'acct_123',
              onEdit: vi.fn()
            }
          ],
          credentialFields: [],
          authTypeItems,
          isChatGptAuth: false,
          showAuthTypeSelect: false,
          showCredentialFields: false,
          submitLabel: 'Create',
          submitDisabled: false,
          showForm: signal(false),
          editId: signal(null),
          providerKey: signal(''),
          authType: signal('api_key'),
          displayName: signal(''),
          chatGptAttempt: signal(null),
          formError: signal(null),
          formSubmitting: signal(false),
          deleteConfirmId: signal(null),
          deleteError: signal(null)
        }}
        status={{ loading: false, catalogLoading: false, error: null }}
        actions={actions}
      />
    );

    expect(getByText('ChatGPT Pro/Plus')).toBeDefined();
    expect(getByText('acct_123')).toBeDefined();
  });
});
