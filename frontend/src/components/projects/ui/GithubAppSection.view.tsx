import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';

interface GithubAppSectionProps {
  installation: { id: number; accountLogin: string; createdAt: string } | undefined;
  connectUrl: string;
  disconnectPending: boolean;
  onDisconnect: (id: number) => void;
}

export const GithubAppSection = ({
  installation,
  connectUrl,
  disconnectPending,
  onDisconnect
}: GithubAppSectionProps): JSX.Element => {
  if (!installation) {
    return (
      <div class='flex flex-col gap-2 p-4 border border-white/10 bg-bg-surface'>
        <span class='text-text-muted text-xs'>
          Connect a GitHub App to enable private repo cloning and PR creation.
        </span>
        <a
          href={connectUrl}
          target='_blank'
          rel='noopener noreferrer'
          class='text-sm text-text-primary underline hover:opacity-80'
        >
          Connect GitHub App
        </a>
      </div>
    );
  }

  return (
    <div class='flex flex-col gap-2 p-4 border border-white/10 bg-bg-surface'>
      <div class='flex items-center justify-between'>
        <span class='text-xs text-success'>
          GitHub App connected &mdash; {installation.accountLogin}
        </span>
        <Button
          type='button'
          variant='secondary'
          disabled={disconnectPending}
          onClick={() => onDisconnect(installation.id)}
        >
          {disconnectPending ? 'Disconnecting...' : 'Disconnect'}
        </Button>
      </div>
    </div>
  );
};
