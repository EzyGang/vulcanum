import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  exchangeAuthCode,
  getAuthMode,
  getGithubLoginUrl
} from '../../../services/auth/auth.service';
import { acceptTeamInvite, previewTeamInvite } from '../../../services/teams/teams.service';
import {
  acceptToken,
  accessToken,
  loadSession,
  setSelectedTeamId
} from '../../../stores/auth.store';
import { invalidate } from '../../../utils/api/query/client';

interface UseInviteAcceptArgs {
  token: string;
}

export type InviteAcceptMode =
  | 'loading'
  | 'ready'
  | 'auth-required'
  | 'single-user'
  | 'accepted'
  | 'invalid';

export const useInviteAccept = ({ token }: UseInviteAcceptArgs) => {
  const [_, setLocation] = useLocation();
  const mode = useSignal<InviteAcceptMode>('loading');
  const error = useSignal<string | null>(null);
  const expiresAt = useSignal<string | null>(null);
  const accepting = useSignal(false);
  const exchangedCode = useSignal(false);

  const acceptInvite = useCallback(async () => {
    accepting.value = true;
    error.value = null;
    try {
      const result = await acceptTeamInvite(token);
      setSelectedTeamId(result.teamId);
      invalidate();
      await loadSession();
      mode.value = 'accepted';
      setLocation('/');
    } catch (err) {
      mode.value = 'invalid';
      error.value = err instanceof Error ? err.message : 'Invalid or expired invite';
    } finally {
      accepting.value = false;
    }
  }, [token]);

  useEffect(() => {
    let cancelled = false;

    const loadInvite = async () => {
      try {
        const [authMode, preview] = await Promise.all([getAuthMode(), previewTeamInvite(token)]);
        if (cancelled) return;

        expiresAt.value = preview.expiresAt;
        if (authMode.isSingleUser) {
          mode.value = 'single-user';
          return;
        }

        mode.value = accessToken.value ? 'ready' : 'auth-required';
      } catch (err) {
        if (cancelled) return;
        mode.value = 'invalid';
        error.value = err instanceof Error ? err.message : 'Invalid or expired invite';
      }
    };

    loadInvite();

    return () => {
      cancelled = true;
    };
  }, [token, accessToken.value]);

  useEffect(() => {
    if (exchangedCode.value) return;

    const code = new URLSearchParams(window.location.search).get('code');
    if (!code) return;

    exchangedCode.value = true;
    accepting.value = true;
    exchangeAuthCode(code)
      .then((tokenPair) => acceptToken(tokenPair.accessToken, true, tokenPair.refreshToken))
      .then(() => acceptInvite())
      .catch((err) => {
        mode.value = 'invalid';
        error.value = err instanceof Error ? err.message : 'GitHub login failed';
        accepting.value = false;
      });
  }, [acceptInvite, exchangedCode.value]);

  const handleGithubLogin = useCallback(() => {
    window.location.href = getGithubLoginUrl(`/invites/${token}`);
  }, [token]);

  const handleAccept = useCallback(() => {
    acceptInvite();
  }, [acceptInvite]);

  return {
    data: {
      expiresAt: expiresAt.value
    },
    status: {
      mode: mode.value,
      error: error.value,
      accepting: accepting.value
    },
    actions: {
      onGithubLogin: handleGithubLogin,
      onAccept: handleAccept
    }
  };
};
