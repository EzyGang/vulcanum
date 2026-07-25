import { useEffect } from 'preact/hooks';
import { getAuthMode } from '../../../services/auth/auth.service';
import {
  disconnectInstallation,
  type GithubAuthUrlResponse,
  getAuthUrl,
  getReviewIdentityAuthUrl,
  listInstallations,
  listRepos
} from '../../../services/github/github.service';
import { queryClient } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

const openGitHubFlow = async (requestUrl: () => Promise<GithubAuthUrlResponse>): Promise<void> => {
  const flowWindow = window.open('', '_blank');

  try {
    const { url } = await requestUrl();
    if (flowWindow) {
      flowWindow.location.href = url;
      return;
    }
    window.location.href = url;
  } catch {
    flowWindow?.close();
  }
};

export const useGitHubApp = () => {
  const { data: authMode } = useApiQuery(['auth-mode'], getAuthMode);
  const {
    data: installations = [],
    isLoading: installationLoading,
    isFetching: installationRefreshing,
    error: installationError,
    refetch
  } = useApiQuery(['github-installations'], listInstallations, {
    retry: false
  });
  const installationIds = installations.map(({ id }) => id).join(',');

  const {
    data: repos = [],
    isLoading: reposLoading,
    error: reposError
  } = useApiQuery(['github-repos'], () => listRepos().then((r) => r.map((repo) => repo.fullName)), {
    enabled: installations.length > 0,
    retry: false
  });

  useEffect(() => {
    if (installations.length === 0) {
      return;
    }

    queryClient.invalidateQueries({ queryKey: ['github-repos'], refetchType: 'active' });
  }, [installationIds]);

  const connectMutation = useApiMutation(getAuthUrl);
  const linkIdentityMutation = useApiMutation(getReviewIdentityAuthUrl);
  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['github-repos'], refetchType: 'active' });
      refetch();
    }
  });

  const onConnect = () => openGitHubFlow(() => connectMutation.mutateAsync(undefined));
  const onLinkReviewIdentity = () =>
    openGitHubFlow(() => linkIdentityMutation.mutateAsync(undefined));
  const onRefresh = (): void => {
    refetch();
  };
  const installationItems = installations.map((installation) => ({
    installation,
    disconnectPending:
      disconnectMutation.isPending && disconnectMutation.variables === installation.id,
    onDisconnect: () => disconnectMutation.mutate(installation.id)
  }));
  const reviewIdentityLogin = installations.find(
    (installation) => installation.reviewIdentityLogin
  )?.reviewIdentityLogin;
  const identityPanel =
    installations.length > 0 && authMode?.isSingleUser
      ? {
          statusText: reviewIdentityLogin
            ? `@${reviewIdentityLogin} can start reviews from PR comments.`
            : 'Link the GitHub account allowed to start reviews from PR comments.',
          actionLabel: linkIdentityMutation.isPending
            ? 'Opening GitHub...'
            : reviewIdentityLogin
              ? 'Change account'
              : 'Link account'
        }
      : null;

  return {
    data: {
      installationItems,
      repos,
      identityPanel
    },
    status: {
      isLoading: installationLoading,
      isRefreshing: installationRefreshing,
      reposLoading,
      identityLinkPending: linkIdentityMutation.isPending,
      errorMessage:
        installationError?.message ??
        reposError?.message ??
        connectMutation.error?.message ??
        linkIdentityMutation.error?.message ??
        null
    },
    actions: {
      onConnect,
      onLinkReviewIdentity,
      onRefresh
    }
  };
};
