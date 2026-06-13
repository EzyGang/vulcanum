import { fireEvent, render, waitFor } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/auth/auth.service', async () => {
  const actual = await vi.importActual<typeof import('../services/auth/auth.service')>(
    '../services/auth/auth.service'
  );

  return {
    ...actual,
    exchangeAuthCode: vi.fn(),
    getAuthMode: vi.fn(),
    getMe: vi.fn(),
    instanceLogin: vi.fn()
  };
});

vi.mock('../services/teams/teams.service', () => ({
  acceptTeamInvite: vi.fn(),
  previewTeamInvite: vi.fn()
}));

const authStore = vi.hoisted(() => {
  const accessToken = { value: null as string | null };
  const selectedTeamId = { value: null as string | null };

  return {
    acceptToken: vi.fn(async () => undefined),
    accessToken,
    clearAuthState: vi.fn(() => {
      accessToken.value = null;
      selectedTeamId.value = null;
    }),
    loadSession: vi.fn(async () => undefined),
    selectedTeamId,
    setSelectedTeamId: vi.fn((teamId: string) => {
      selectedTeamId.value = teamId;
    })
  };
});

vi.mock('../stores/auth.store', () => authStore);

const setLocation = vi.fn();
vi.mock('wouter-preact', () => ({
  useLocation: () => ['/', setLocation]
}));

vi.mock('../utils/api/query/client', () => ({
  invalidate: vi.fn()
}));

import { useInviteAccept } from '../components/invites/hooks/useInviteAccept.hook';
import { InviteAcceptView } from '../components/invites/ui/InviteAccept.view';
import { exchangeAuthCode, getAuthMode, getGithubLoginUrl } from '../services/auth/auth.service';
import { acceptTeamInvite, previewTeamInvite } from '../services/teams/teams.service';
import { accessToken, clearAuthState, selectedTeamId } from '../stores/auth.store';
import { invalidate } from '../utils/api/query/client';

const HookHarness = ({ token }: { token: string }) => {
  const invite = useInviteAccept({ token });

  return (
    <div>
      <span data-testid='mode'>{invite.status.mode}</span>
      <span data-testid='error'>{invite.status.error}</span>
      <button type='button' onClick={invite.actions.onAccept}>
        accept
      </button>
      <button type='button' onClick={invite.actions.onGithubLogin}>
        login
      </button>
    </div>
  );
};

describe('InviteAccept', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.history.pushState({}, '', '/invites/invite-token');
    clearAuthState();
    vi.mocked(getAuthMode).mockResolvedValue({ isSingleUser: false });
    vi.mocked(previewTeamInvite).mockResolvedValue({ expiresAt: '2030-01-01T00:00:00Z' });
  });

  it('renders unauthenticated invite state', () => {
    const { getByText } = render(
      <InviteAcceptView
        data={{ expiresAt: '2030-01-01T00:00:00Z' }}
        status={{ mode: 'auth-required', error: null, accepting: false }}
        actions={{ onGithubLogin: vi.fn(), onAccept: vi.fn() }}
      />
    );

    expect(getByText('Join a Vulcanum team')).toBeDefined();
    expect(getByText('Sign in with GitHub')).toBeDefined();
    expect(getByText(/2030-01-01/)).toBeDefined();
  });

  it('renders invalid invite state', () => {
    const { getByText } = render(
      <InviteAcceptView
        data={{ expiresAt: null }}
        status={{ mode: 'invalid', error: null, accepting: false }}
        actions={{ onGithubLogin: vi.fn(), onAccept: vi.fn() }}
      />
    );

    expect(getByText(/invalid or expired/)).toBeDefined();
  });

  it('constructs GitHub login return URL', () => {
    expect(getGithubLoginUrl('/invites/invite-token')).toBe(
      '/api/v1/auth/github/start?return_to=%2Finvites%2Finvite-token'
    );
  });

  it('accepts invite for an authenticated user and redirects', async () => {
    accessToken.value = 'access-token';
    vi.mocked(acceptTeamInvite).mockResolvedValue({ teamId: 'joined-team' });

    const { getByText } = render(<HookHarness token='invite-token' />);

    await waitFor(() => expect(getByText('ready')).toBeDefined());
    fireEvent.click(getByText('accept'));

    await waitFor(() => expect(acceptTeamInvite).toHaveBeenCalledWith('invite-token'));
    expect(selectedTeamId.value).toBe('joined-team');
    expect(invalidate).toHaveBeenCalled();
    expect(authStore.loadSession).toHaveBeenCalled();
    expect(setLocation).toHaveBeenCalledWith('/');
  });

  it('exchanges callback code before accepting invite', async () => {
    window.history.pushState({}, '', '/invites/invite-token?code=callback-code');
    vi.mocked(exchangeAuthCode).mockResolvedValue({
      accessToken: 'new-access-token',
      refreshToken: 'new-refresh-token',
      refreshExpiresAt: '2030-01-01T00:00:00Z'
    });
    vi.mocked(acceptTeamInvite).mockResolvedValue({ teamId: 'joined-team' });

    render(<HookHarness token='invite-token' />);

    await waitFor(() => expect(exchangeAuthCode).toHaveBeenCalledWith('callback-code'));
    await waitFor(() => expect(acceptTeamInvite).toHaveBeenCalledWith('invite-token'));
    expect(selectedTeamId.value).toBe('joined-team');
    expect(setLocation).toHaveBeenCalledWith('/');
  });

  it('shows invalid mode when preview fails', async () => {
    vi.mocked(previewTeamInvite).mockRejectedValue(new Error('Invalid or expired invite'));

    const { getByText } = render(<HookHarness token='bad-token' />);

    await waitFor(() => expect(getByText('invalid')).toBeDefined());
    expect(getByText('Invalid or expired invite')).toBeDefined();
  });
});
