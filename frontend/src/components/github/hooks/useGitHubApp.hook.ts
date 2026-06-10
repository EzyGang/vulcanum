import {
  disconnectInstallation,
  getAuthUrl,
  getInstallation,
  listRepos
} from '../../../services/github/github.service';
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

  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      refetch();
    }
  });

  const onConnect = async () => {
    const { url } = await getAuthUrl();
    window.open(url, '_blank');
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
