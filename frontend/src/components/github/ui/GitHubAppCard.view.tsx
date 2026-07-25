import { IconBrandGithub, IconPlugConnected, IconRefresh, IconUnlink } from '@tabler/icons-react';
import type { JSX } from 'preact';
import type { GithubInstallation } from '../../../types/github';
import { ActionIconButton } from '../../shared/ui/ActionIconButton.view';
import { Button } from '../../shared/ui/Button.view';
import { Card } from '../../shared/ui/Card.view';
import { ErrorBanner } from '../../shared/ui/ErrorBanner.view';

interface GitHubAppCardViewProps {
  data: {
    installationItems: {
      installation: GithubInstallation;
      disconnectPending: boolean;
      onDisconnect: () => void;
    }[];
  };
  identityPanel: JSX.Element | null;
  status: {
    isLoading: boolean;
    isRefreshing: boolean;
    errorMessage: string | null;
  };
  actions: {
    onConnect: () => void;
    onRefresh: () => void;
  };
}

export const GitHubAppCardView = ({
  data: { installationItems },
  identityPanel,
  status: { isLoading, isRefreshing, errorMessage },
  actions: { onConnect, onRefresh }
}: GitHubAppCardViewProps): JSX.Element => {
  const connected = installationItems.length > 0;

  return (
    <Card class='flex flex-col gap-4'>
      <div class='flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3'>
        <div class='flex items-center gap-3'>
          <span class='inline-flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-text-primary'>
            <IconBrandGithub size={16} stroke={1.75} aria-hidden='true' />
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
          <ActionIconButton
            label='Refresh GitHub app status'
            onClick={onRefresh}
            disabled={isRefreshing}
          >
            <IconRefresh size={16} stroke={1.75} aria-hidden='true' />
          </ActionIconButton>

          <Button variant='secondary' onClick={onConnect}>
            <span class='inline-flex items-center gap-2'>
              <IconPlugConnected size={16} stroke={1.75} aria-hidden='true' />
              {connected ? 'Connect another account' : 'Connect'}
            </span>
          </Button>
        </div>
      </div>

      {errorMessage && <ErrorBanner message={errorMessage} />}

      {connected && (
        <div class='flex flex-col border border-border-base bg-bg-input'>
          {installationItems.map(({ installation, disconnectPending, onDisconnect }) => (
            <div
              key={installation.id}
              class='flex items-center justify-between gap-3 border-b border-border-base px-3 py-2 last:border-b-0'
            >
              <div class='flex items-center gap-2'>
                <span class='text-text-muted text-xs'>Account:</span>
                <span class='text-text-primary text-sm font-mono'>{installation.accountLogin}</span>
              </div>
              <ActionIconButton
                label={`Disconnect ${installation.accountLogin}`}
                variant='danger'
                onClick={onDisconnect}
                disabled={disconnectPending}
              >
                <IconUnlink size={16} stroke={1.75} aria-hidden='true' />
              </ActionIconButton>
            </div>
          ))}
        </div>
      )}
      {identityPanel}
    </Card>
  );
};
