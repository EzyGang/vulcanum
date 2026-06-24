import { QueryClientProvider } from '@tanstack/react-query';
import { cleanup, fireEvent, render, waitFor } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useModelProviders } from '../components/model-providers/hooks/useModelProviders.hook';
import * as modelProvidersService from '../services/model-providers/model-providers.service';
import type { CatalogResponse, ModelProviderConfig } from '../types/modelProviders';
import { queryClient } from '../utils/api/query/client';

vi.mock('../services/model-providers/model-providers.service', () => ({
  cancelChatGptAuth: vi.fn(),
  createModelProvider: vi.fn(),
  deleteModelProvider: vi.fn(),
  getChatGptAuthStatus: vi.fn(),
  getModelProviderCatalog: vi.fn(),
  listModelProviders: vi.fn(),
  startChatGptAuth: vi.fn(),
  updateModelProvider: vi.fn()
}));

type ModelProvidersState = ReturnType<typeof useModelProviders>;

const catalog: CatalogResponse = {
  providers: [
    {
      id: 'openai',
      name: 'OpenAI',
      doc: '',
      env: ['OPENAI_API_KEY'],
      models: []
    }
  ]
};

const connectedProvider: ModelProviderConfig = {
  id: 'provider-1',
  providerKey: 'openai',
  authType: 'chatgpt_oauth',
  displayName: 'OpenAI ChatGPT Pro/Plus',
  credentials: {},
  oauthMetadata: { accountId: 'acct_123' },
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z'
};

const HookProbe = ({ onValue }: { onValue: (value: ModelProvidersState) => void }) => {
  const value = useModelProviders();
  onValue(value);

  return (
    <form onSubmit={value.actions.onSave}>
      <button type='button' data-testid='create-button' onClick={value.actions.onShowCreate}>
        Create
      </button>
      <button
        type='button'
        data-testid='provider-button'
        onClick={() => value.actions.onProviderChange('openai')}
      >
        OpenAI
      </button>
      <button
        type='button'
        data-testid='auth-type-button'
        onClick={() => value.actions.onAuthTypeChange('chatgpt_oauth')}
      >
        ChatGPT OAuth
      </button>
      <button type='submit' data-testid='save-button'>
        Save
      </button>
      <button
        type='button'
        data-testid='cancel-chatgpt-button'
        onClick={value.actions.onCancelChatGptAuth}
      >
        Cancel ChatGPT
      </button>
      <span data-testid='attempt-id'>{value.data.chatGptAttempt.value?.attemptId ?? ''}</span>
      <span data-testid='form-error'>{value.data.formError.value ?? ''}</span>
      <span data-testid='poll-interval'>
        {value.data.chatGptAttempt.value?.pollIntervalSeconds ?? ''}
      </span>
      <span data-testid='show-form'>{String(value.data.showForm.value)}</span>
      <span data-testid='submit-label'>{value.data.submitLabel}</span>
    </form>
  );
};

describe('useModelProviders ChatGPT OAuth polling', () => {
  beforeEach(() => {
    queryClient.clear();
    vi.mocked(modelProvidersService.getModelProviderCatalog).mockResolvedValue(catalog);
    vi.mocked(modelProvidersService.listModelProviders).mockResolvedValue([]);
    vi.mocked(modelProvidersService.cancelChatGptAuth).mockResolvedValue(undefined);
    vi.mocked(modelProvidersService.startChatGptAuth).mockResolvedValue({
      attemptId: 'attempt-1',
      verificationUri: 'https://auth.example/device',
      userCode: 'ABCD-EFGH',
      expiresAt: new Date(Date.now() + 600_000).toISOString(),
      pollIntervalSeconds: 5
    });
  });

  afterEach(() => {
    cleanup();
    queryClient.clear();
    vi.clearAllMocks();
  });

  it('keeps pending attempts active and adopts server poll interval changes', async () => {
    vi.mocked(modelProvidersService.getChatGptAuthStatus).mockResolvedValue({
      status: 'pending',
      pollIntervalSeconds: 9
    });
    const current: { value?: ModelProvidersState } = {};
    const { getByTestId } = render(
      <QueryClientProvider client={queryClient}>
        <HookProbe onValue={(value) => (current.value = value)} />
      </QueryClientProvider>
    );

    startChatGptAuth(getByTestId);

    await waitFor(() =>
      expect(modelProvidersService.getChatGptAuthStatus).toHaveBeenCalledTimes(1)
    );
    await waitFor(() => expect(getByTestId('attempt-id').textContent).toBe('attempt-1'));
    await waitFor(() => expect(getByTestId('poll-interval').textContent).toBe('9'));
    expect(current.value?.data.submitLabel).toBe('Waiting for Login');
  });

  it('resets the form when the server completes the OAuth attempt', async () => {
    vi.mocked(modelProvidersService.getChatGptAuthStatus).mockResolvedValue({
      status: 'complete',
      provider: connectedProvider
    });
    const { getByTestId } = render(
      <QueryClientProvider client={queryClient}>
        <HookProbe onValue={() => undefined} />
      </QueryClientProvider>
    );

    startChatGptAuth(getByTestId);

    await waitFor(() =>
      expect(modelProvidersService.getChatGptAuthStatus).toHaveBeenCalledTimes(1)
    );
    await waitFor(() => expect(getByTestId('show-form').textContent).toBe('false'));
    expect(getByTestId('attempt-id').textContent).toBe('');
  });

  it.each([
    { status: 'expired' as const, error: 'Device code expired' },
    { status: 'failed' as const, error: 'Access denied' }
  ])('clears $status attempts and shows the server error', async ({ status, error }) => {
    vi.mocked(modelProvidersService.getChatGptAuthStatus).mockResolvedValue({ status, error });
    const { getByTestId } = render(
      <QueryClientProvider client={queryClient}>
        <HookProbe onValue={() => undefined} />
      </QueryClientProvider>
    );

    startChatGptAuth(getByTestId);

    await waitFor(() =>
      expect(modelProvidersService.getChatGptAuthStatus).toHaveBeenCalledTimes(1)
    );
    await waitFor(() => expect(getByTestId('attempt-id').textContent).toBe(''));
    expect(getByTestId('form-error').textContent).toBe(error);
    expect(getByTestId('submit-label').textContent).toBe('Start Device Login');
  });

  it('cancels the active OAuth attempt and clears local attempt state', async () => {
    vi.mocked(modelProvidersService.getChatGptAuthStatus).mockResolvedValue({
      status: 'pending',
      pollIntervalSeconds: 5
    });
    const { getByTestId } = render(
      <QueryClientProvider client={queryClient}>
        <HookProbe onValue={() => undefined} />
      </QueryClientProvider>
    );

    startChatGptAuth(getByTestId);
    await waitFor(() => expect(getByTestId('attempt-id').textContent).toBe('attempt-1'));
    fireEvent.click(getByTestId('cancel-chatgpt-button'));

    await waitFor(() =>
      expect(modelProvidersService.cancelChatGptAuth).toHaveBeenCalledWith('attempt-1')
    );
    await waitFor(() => expect(getByTestId('attempt-id').textContent).toBe(''));
  });
});

const startChatGptAuth = (getByTestId: (testId: string) => HTMLElement): void => {
  fireEvent.click(getByTestId('create-button'));
  fireEvent.click(getByTestId('provider-button'));
  fireEvent.click(getByTestId('auth-type-button'));
  fireEvent.click(getByTestId('save-button'));
};
