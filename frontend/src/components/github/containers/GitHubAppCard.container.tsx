import type { JSX } from 'preact';
import { useGitHubApp } from '../hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../ui/GitHubAppCard.view';

export const GitHubAppCardContainer = (): JSX.Element => {
  const { data, status, actions } = useGitHubApp();
  return <GitHubAppCardView data={data} status={status} actions={actions} />;
};
