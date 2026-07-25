export interface RepoInfo {
  owner: string;
  name: string;
  fullName: string;
  installationId: number;
  accountLogin: string;
}

export interface GithubInstallation {
  id: number;
  accountLogin: string;
  reviewIdentityUserId?: string | null;
  reviewIdentityLogin?: string | null;
  createdAt: string;
}
