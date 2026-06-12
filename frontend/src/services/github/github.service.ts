import type { GithubInstallation, RepoInfo } from '../../types/github';
import { del, get } from '../../utils/api/request';

interface GithubAuthUrlResponse {
  url: string;
}

export const getAuthUrl = () => get<GithubAuthUrlResponse>('/github/auth-url');

export const getInstallation = () => get<GithubInstallation | null>('/github/installation');

export const listRepos = () => get<RepoInfo[]>('/github/repos');

export const disconnectInstallation = (id: number) => del(`/github/installation/${id}`);
