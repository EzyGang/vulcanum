import { useEffect } from 'preact/hooks';
import { getAuthMode } from '../../../services/auth/auth.service';
import {
  disconnectInstallation,
  type GithubAuthUrlResponse,
  getAuthUrl,
  getInstallation,
  getReviewIdentityAuthUrl,
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
    data: installation,
    isLoading: installationLoading,
    isFetching: installationRefreshing,
    error: installationError,
    refetch
  } = useApiQuery(['github-installation'], () => getInstallation(), {
    retry: false
  });

  const {
    data: repos = [],
    isLoading: reposLoading,
    error: reposError
  } = useApiQuery(['github-repos'], () => listRepos().then((r) => r.map((repo) => repo.fullName)), {
    enabled: !!installation,
    retry: false
  });

  useEffect(() => {
    if (!installation) {
      return;
    }

    queryClient.invalidateQueries({ queryKey: ['github-repos'], refetchType: 'active' });
  }, [installation?.id]);

  const connectMutation = useApiMutation(getAuthUrl);
  const linkIdentityMutation = useApiMutation(getReviewIdentityAuthUrl);
  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: ['github-repos'] });
      refetch();
    }
  });

  const onConnect = () => openGitHubFlow(() => connectMutation.mutateAsync(undefined));
  const onLinkReviewIdentity = () =>
    openGitHubFlow(() => linkIdentityMutation.mutateAsync(undefined));
  const onDisconnect = (): void => {
    if (installation) {
      disconnectMutation.mutate(installation.id);
    }
  };
  const onRefresh = (): void => {
    refetch();
  };
  const reviewIdentityLogin = installation?.reviewIdentityLogin;
  const identityPanelVisible = !!installation && (authMode?.isSingleUser ?? false);
  const identityStatusText = reviewIdentityLogin
    ? `@${reviewIdentityLogin} can start reviews from PR comments.`
    : 'Link the GitHub account allowed to start reviews from PR comments.';
  const identityActionLabel = linkIdentityMutation.isPending
    ? 'Opening GitHub...'
    : reviewIdentityLogin
      ? 'Change account'
      : 'Link account';

  return {
    data: {
      installation: installation ?? null,
      repos,
      identityPanelVisible,
      identityStatusText,
      identityActionLabel
    },
    status: {
      isLoading: installationLoading,
      isRefreshing: installationRefreshing,
      reposLoading,
      disconnectPending: disconnectMutation.isPending,
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
      onRefresh,
      onDisconnect
    }
  };
};
