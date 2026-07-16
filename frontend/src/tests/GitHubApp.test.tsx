import { QueryClientProvider } from '@tanstack/react-query';
import { render, waitFor } from '@testing-library/preact';
import type { JSX } from 'preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/github/github.service', () => ({
  disconnectInstallation: vi.fn(),
  getAuthUrl: vi.fn(),
  getInstallation: vi.fn(),
  listRepos: vi.fn()
}));

import { useGitHubApp } from '../components/github/hooks/useGitHubApp.hook';
import { getInstallation, listRepos } from '../services/github/github.service';
import { queryClient } from '../utils/api/query/client';

const GitHubReposHarness = (): JSX.Element => {
  const { repos } = useGitHubApp();
  return <div>{repos.join(',')}</div>;
};

beforeEach(() => {
  queryClient.clear();
  vi.clearAllMocks();
});

describe('useGitHubApp', () => {
  it('refreshes a fresh empty repository cache when an installation appears', async () => {
    vi.mocked(getInstallation).mockResolvedValue({
      id: 42,
      accountLogin: 'vulcanum',
      createdAt: '2026-07-16T00:00:00Z'
    });
    vi.mocked(listRepos).mockResolvedValue([
      { owner: 'vulcanum', name: 'core', fullName: 'vulcanum/core' }
    ]);
    queryClient.setQueryData(['github-repos'], []);

    const view = render(
      <QueryClientProvider client={queryClient}>
        <GitHubReposHarness />
      </QueryClientProvider>
    );

    await waitFor(() => expect(listRepos).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(view.getByText('vulcanum/core')).toBeTruthy());
  });
});
