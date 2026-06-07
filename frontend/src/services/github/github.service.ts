import type { GithubInstallation, RepoInfo } from '../../types/github';
import { del, get } from '../../utils/api/request';

export const getAuthUrl = () => '/api/v1/github/auth';

export const getInstallation = () => get<GithubInstallation | null>('/github/installation');

export const listRepos = () => get<RepoInfo[]>('/github/repos');

export const disconnectInstallation = (id: number) => del(`/github/installation/${id}`);
