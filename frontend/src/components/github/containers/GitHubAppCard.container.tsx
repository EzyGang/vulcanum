import type { JSX } from 'preact';
import { useGitHubApp } from '../hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../ui/GitHubAppCard.view';

export const GitHubAppCardContainer = (): JSX.Element => {
  const {
    installation,
    installationLoading,
    disconnectPending,
    onConnect,
    disconnectInstallation,
    refetch
  } = useGitHubApp();

  return (
    <GitHubAppCardView
      installation={installation ?? null}
      isLoading={installationLoading}
      disconnectPending={disconnectPending}
      onConnect={onConnect}
      onRefresh={refetch}
      onDisconnect={() => {
        if (installation) {
          disconnectInstallation(installation.id);
        }
      }}
    />
  );
};
