import type { GithubInstallation, RepoInfo } from '../../types/github';
import { del, get } from '../../utils/api/request';

export interface GithubAuthUrlResponse {
  url: string;
}

export const getAuthUrl = () => get<GithubAuthUrlResponse>('/github/auth-url');
export const getReviewIdentityAuthUrl = () =>
  get<GithubAuthUrlResponse>('/auth/github/link-url?return_to=/settings%3Ftab%3Dgithub');

export const getInstallation = () => get<GithubInstallation | null>('/github/installation');

export const listRepos = () => get<RepoInfo[]>('/github/repos');

export const disconnectInstallation = (id: number) => del(`/github/installation/${id}`);
