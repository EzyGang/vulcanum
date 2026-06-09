import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Card } from '../../shared/ui/Card.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';

interface GitHubAppCardViewProps {
  data: {
    installation: { id: number; accountLogin: string } | null;
  };
  status: {
    isLoading: boolean;
    isRefreshing: boolean;
    disconnectPending: boolean;
    errorMessage: string | null;
  };
  actions: {
    onConnect: () => void;
    onRefresh: () => void;
    onDisconnect: () => void;
  };
}

export const GitHubAppCardView = ({
  data: { installation },
  status: { isLoading, isRefreshing, disconnectPending, errorMessage },
  actions: { onConnect, onRefresh, onDisconnect }
}: GitHubAppCardViewProps): JSX.Element => {
  const connected = !!installation;

  return (
    <Card class='flex flex-col gap-4'>
      <div class='flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3'>
        <div class='flex items-center gap-3'>
          <span class='text-text-primary text-sm font-semibold uppercase tracking-wider'>
            GitHub App
          </span>
          {isLoading ? (
            <span class='text-text-muted text-xs animate-pulse'>Loading...</span>
          ) : connected ? (
            <span class='text-success text-xs uppercase tracking-wider px-2 py-0.5 border border-success-border bg-success-bg'>
              Connected
            </span>
          ) : (
            <span class='text-text-muted text-xs uppercase tracking-wider px-2 py-0.5 border border-border-base bg-bg-hover'>
              Not Connected
            </span>
          )}
          {isRefreshing && !isLoading && (
            <span class='text-text-muted text-xs animate-pulse'>Refreshing...</span>
          )}
        </div>

        <div class='flex items-center gap-2'>
          <Button variant='ghost' onClick={onRefresh} disabled={isRefreshing}>
            Refresh
          </Button>

          {connected ? (
            <Button variant='ghost-danger' onClick={onDisconnect} disabled={disconnectPending}>
              {disconnectPending ? 'Disconnecting...' : 'Disconnect'}
            </Button>
          ) : (
            <Button variant='secondary' onClick={onConnect}>
              Connect
            </Button>
          )}
        </div>
      </div>

      {errorMessage && <ErrorBanner message={errorMessage} />}

      {connected && installation && (
        <div class='flex items-center gap-2'>
          <span class='text-text-muted text-xs'>Account:</span>
          <span class='text-text-primary text-sm font-mono'>{installation.accountLogin}</span>
        </div>
      )}
    </Card>
  );
};
