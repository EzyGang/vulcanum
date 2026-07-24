import type { JSX } from 'preact';
import { useGitHubApp } from '../hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../ui/GitHubAppCard.view';
import { GitHubReviewIdentityPanel } from '../ui/GitHubReviewIdentityPanel.view';

export const GitHubAppCardContainer = (): JSX.Element => {
  const { data, status, actions } = useGitHubApp();
  const identityPanel = data.identityPanel ? (
    <GitHubReviewIdentityPanel
      {...data.identityPanel}
      actionPending={status.identityLinkPending}
      onAction={actions.onLinkReviewIdentity}
    />
  ) : null;

  return (
    <GitHubAppCardView
      data={data}
      identityPanel={identityPanel}
      status={status}
      actions={actions}
    />
  );
};
