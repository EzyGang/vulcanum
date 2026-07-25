import { QueryClientProvider } from '@tanstack/react-query';
import { render, waitFor } from '@testing-library/preact';
import type { JSX } from 'preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../services/auth/auth.service', () => ({
  getAuthMode: vi.fn()
}));

vi.mock('../services/github/github.service', () => ({
  disconnectInstallation: vi.fn(),
  getAuthUrl: vi.fn(),
  listInstallations: vi.fn(),
  getReviewIdentityAuthUrl: vi.fn(),
  listRepos: vi.fn()
}));

import { useGitHubApp } from '../components/github/hooks/useGitHubApp.hook';
import { GitHubAppCardView } from '../components/github/ui/GitHubAppCard.view';
import { getAuthMode } from '../services/auth/auth.service';
import { listInstallations, listRepos } from '../services/github/github.service';
import { queryClient } from '../utils/api/query/client';

const GitHubReposHarness = (): JSX.Element => {
  const { data } = useGitHubApp();
  return <div>{data.repos.join(',')}</div>;
};

beforeEach(() => {
  queryClient.clear();
  vi.clearAllMocks();
  vi.mocked(getAuthMode).mockResolvedValue({ isSingleUser: true });
});

describe('useGitHubApp', () => {
  it('loads repositories from every installation', async () => {
    vi.mocked(listInstallations).mockResolvedValue([
      {
        id: 42,
        accountLogin: 'vulcanum',
        createdAt: '2026-07-16T00:00:00Z'
      },
      {
        id: 43,
        accountLogin: 'galtozzy',
        createdAt: '2026-07-17T00:00:00Z'
      }
    ]);
    vi.mocked(listRepos).mockResolvedValue([
      {
        owner: 'vulcanum',
        name: 'core',
        fullName: 'vulcanum/core',
        installationId: 42,
        accountLogin: 'vulcanum'
      },
      {
        owner: 'galtozzy',
        name: 'personal',
        fullName: 'galtozzy/personal',
        installationId: 43,
        accountLogin: 'galtozzy'
      }
    ]);

    const view = render(
      <QueryClientProvider client={queryClient}>
        <GitHubReposHarness />
      </QueryClientProvider>
    );

    await waitFor(() => expect(listRepos).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(view.getByText('vulcanum/core,galtozzy/personal')).toBeTruthy());
  });
});

describe('GitHubAppCardView', () => {
  it('shows every installation and keeps the connect action available', () => {
    const view = render(
      <GitHubAppCardView
        data={{
          installationItems: [
            {
              installation: {
                id: 42,
                accountLogin: 'vulcanum',
                createdAt: '2026-07-16T00:00:00Z'
              },
              disconnectPending: false,
              onDisconnect: vi.fn()
            },
            {
              installation: {
                id: 43,
                accountLogin: 'galtozzy',
                createdAt: '2026-07-17T00:00:00Z'
              },
              disconnectPending: false,
              onDisconnect: vi.fn()
            }
          ]
        }}
        identityPanel={null}
        status={{
          isLoading: false,
          isRefreshing: false,
          errorMessage: null
        }}
        actions={{
          onConnect: vi.fn(),
          onRefresh: vi.fn()
        }}
      />
    );

    expect(view.getByText('vulcanum')).toBeTruthy();
    expect(view.getByText('galtozzy')).toBeTruthy();
    expect(view.getByText('Connect another account')).toBeTruthy();
  });
});
