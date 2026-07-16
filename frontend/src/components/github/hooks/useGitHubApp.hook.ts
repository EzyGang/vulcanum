import { useEffect } from 'preact/hooks';
import {
  disconnectInstallation,
  getAuthUrl,
  getInstallation,
  listRepos
} from '../../../services/github/github.service';
import { queryClient } from '../../../utils/api/query/client';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useGitHubApp = () => {
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

  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: ['github-repos'] });
      refetch();
    }
  });

  const onConnect = async () => {
    const installWindow = window.open('', '_blank');

    try {
      const { url } = await getAuthUrl();
      if (installWindow) {
        installWindow.location.href = url;
        return;
      }
      window.location.href = url;
    } catch (error) {
      installWindow?.close();
      throw error;
    }
  };

  return {
    installation,
    repos,
    reposLoading,
    reposError,
    installationLoading,
    installationRefreshing,
    installationErrorMessage: installationError?.message ?? reposError?.message ?? null,
    installationError,
    onConnect,
    disconnectInstallation: disconnectMutation.mutateAsync,
    disconnectPending: disconnectMutation.isPending,
    refetch
  };
};
