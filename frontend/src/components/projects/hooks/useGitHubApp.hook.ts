import {
  disconnectInstallation,
  getInstallation,
  listRepos
} from '../../../services/github/github.service';
import { useApiMutation, useApiQuery } from '../../../utils/api/query/hooks';

export const useGitHubApp = () => {
  const { data: installation, refetch } = useApiQuery(
    ['github-installation'],
    () => getInstallation(),
    {
      retry: false,
      refetchOnWindowFocus: false
    }
  );

  const { data: repos = [], isLoading: reposLoading } = useApiQuery(
    ['github-repos'],
    () => listRepos().then((r) => r.map((repo) => repo.fullName)),
    { enabled: !!installation, retry: false }
  );

  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      refetch();
    }
  });

  const connectUrl = '/api/v1/github/auth';

  return {
    installation,
    repos,
    reposLoading,
    connectUrl,
    disconnectInstallation: disconnectMutation.mutateAsync,
    disconnectPending: disconnectMutation.isPending,
    refetch
  };
};
