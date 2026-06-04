import { useSignal } from '@preact/signals';
import { useCallback, useEffect } from 'preact/hooks';
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

  const repos = useSignal<string[]>([]);
  const reposLoading = useSignal(false);
  const reposError = useSignal<string | null>(null);

  const fetchRepos = useCallback(async () => {
    reposLoading.value = true;
    reposError.value = null;
    try {
      const data = await listRepos();
      repos.value = data.map((r) => r.fullName);
    } catch (e) {
      reposError.value = e instanceof Error ? e.message : 'Failed to load repos';
    } finally {
      reposLoading.value = false;
    }
  }, []);

  useEffect(() => {
    if (installation) {
      fetchRepos();
    }
  }, [installation, fetchRepos]);

  const disconnectMutation = useApiMutation((id: number) => disconnectInstallation(id), {
    onSuccess: () => {
      repos.value = [];
      refetch();
    }
  });

  const connectUrl = '/api/v1/github/auth';

  return {
    installation,
    repos,
    reposLoading,
    reposError,
    connectUrl,
    disconnectInstallation: disconnectMutation.mutateAsync,
    disconnectPending: disconnectMutation.isPending,
    refetch,
    fetchRepos
  };
};
