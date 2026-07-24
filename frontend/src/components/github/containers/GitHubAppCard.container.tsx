import type { JSX } from 'preact';
import { useGitHubApp } from '../hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../ui/GitHubAppCard.view';

export const GitHubAppCardContainer = (): JSX.Element => {
  const {
    installation,
    isSingleUser,
    installationLoading,
    installationRefreshing,
    installationErrorMessage,
    disconnectPending,
    identityLinkPending,
    onConnect,
    onLinkReviewIdentity,
    disconnectInstallation,
    refetch
  } = useGitHubApp();

  return (
    <GitHubAppCardView
      data={{ installation: installation ?? null, isSingleUser }}
      status={{
        isLoading: installationLoading,
        isRefreshing: installationRefreshing,
        disconnectPending,
        identityLinkPending,
        errorMessage: installationErrorMessage
      }}
      actions={{
        onConnect,
        onLinkReviewIdentity,
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
