import type { RepoInfo } from '../../types/github';
import { del, get } from '../../utils/api/request';

export const getInstallation = () =>
  get<{ id: number; accountLogin: string; createdAt: string } | null>('/github/installation');
export const listRepos = () => get<RepoInfo[]>('/github/repos');
export const disconnectInstallation = (id: number) => del(`/github/installation/${id}`);
