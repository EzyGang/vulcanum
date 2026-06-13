import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';
import type { InviteAcceptMode } from '../hooks/useInviteAccept.hook';

interface InviteAcceptViewProps {
  data: {
    expiresAt: string | null;
  };
  status: {
    mode: InviteAcceptMode;
    error: string | null;
    accepting: boolean;
  };
  actions: {
    onGithubLogin: () => void;
    onAccept: () => void;
  };
}

export const InviteAcceptView = ({ data, status, actions }: InviteAcceptViewProps): JSX.Element => (
  <main class='min-h-screen bg-bg-page px-6 py-16 text-text-primary'>
    <section class='mx-auto flex max-w-xl flex-col gap-6 border border-border-base bg-bg-card p-6'>
      <div class='flex flex-col gap-3'>
        <span class='text-xs uppercase tracking-wider text-text-muted'>Team Invite</span>
        <h1 class='text-3xl font-semibold tracking-tight'>Join a Vulcanum team</h1>
        <p class='text-sm leading-6 text-text-secondary'>
          This invite lets a GitHub-authenticated Vulcanum user join a team as a member. Team
          details are shown only after authentication.
        </p>
      </div>

      {status.error && <ErrorBanner message={status.error} />}

      {status.mode === 'loading' && <p class='text-sm text-text-muted'>Checking invite...</p>}

      {status.mode === 'invalid' && (
        <div class='border border-border-base bg-bg-panel p-4 text-sm text-text-muted'>
          This invite is invalid or expired. Ask a team owner to generate a new invite link.
        </div>
      )}

      {status.mode === 'single-user' && (
        <div class='border border-border-base bg-bg-panel p-4 text-sm text-text-muted'>
          Link invites require multiuser mode and GitHub OAuth authentication.
        </div>
      )}

      {(status.mode === 'auth-required' || status.mode === 'ready') && (
        <div class='flex flex-col gap-4'>
          {data.expiresAt && (
            <div class='border border-border-base bg-bg-panel p-4 text-sm text-text-muted'>
              Invite expires at <span class='font-mono text-text-secondary'>{data.expiresAt}</span>
            </div>
          )}
          {status.mode === 'auth-required' ? (
            <Button type='button' variant='primary' onClick={actions.onGithubLogin}>
              Sign in with GitHub
            </Button>
          ) : (
            <Button
              type='button'
              variant='primary'
              onClick={actions.onAccept}
              disabled={status.accepting}
            >
              {status.accepting ? 'Joining...' : 'Accept Invite'}
            </Button>
          )}
        </div>
      )}

      {status.mode === 'accepted' && <p class='text-sm text-success'>Invite accepted.</p>}
    </section>
  </main>
);
