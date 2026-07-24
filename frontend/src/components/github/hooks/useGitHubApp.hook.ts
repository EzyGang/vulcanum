import { useEffect } from 'preact/hooks';
import { getAuthMode } from '../../../services/auth/auth.service';
import {
  disconnectInstallation,
  getAuthUrl,
  getInstallation,
  getReviewIdentityAuthUrl,
  listRepos
} from '../../../services/github/github.service';
import { queryClient } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

interface AuthUrlResponse {
  url: string;
}

const openGitHubFlow = async (requestUrl: () => Promise<AuthUrlResponse>): Promise<void> => {
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

  return {
    installation,
    repos,
    reposLoading,
    reposError,
    installationLoading,
    installationRefreshing,
    isSingleUser: authMode?.isSingleUser ?? false,
    installationErrorMessage:
      installationError?.message ??
      reposError?.message ??
      connectMutation.error?.message ??
      linkIdentityMutation.error?.message ??
      null,
    installationError,
    onConnect,
    onLinkReviewIdentity,
    identityLinkPending: linkIdentityMutation.isPending,
    disconnectInstallation: disconnectMutation.mutateAsync,
    disconnectPending: disconnectMutation.isPending,
    refetch
  };
};
