import type { JSX } from 'preact';
import { useGitHubApp } from '../hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../ui/GitHubAppCard.view';

export const GitHubAppCardContainer = (): JSX.Element => {
  const {
    installation,
    installationLoading,
    installationRefreshing,
    installationError,
    reposError,
    disconnectPending,
    onConnect,
    disconnectInstallation,
    refetch
  } = useGitHubApp();

  return (
    <GitHubAppCardView
      data={{ installation: installation ?? null }}
      status={{
        isLoading: installationLoading,
        isRefreshing: installationRefreshing,
        disconnectPending,
        installationError,
        reposError
      }}
      actions={{
        onConnect,
        onRefresh: refetch,
        onDisconnect: () => {
          if (installation) {
            disconnectInstallation(installation.id);
          }
        }
      }}
    />
  );
};
