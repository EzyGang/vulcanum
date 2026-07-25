export interface RepoInfo {
  owner: string;
  name: string;
  fullName: string;
}

export interface GithubInstallation {
  id: number;
  accountLogin: string;
  reviewIdentityUserId?: string | null;
  reviewIdentityLogin?: string | null;
  createdAt: string;
}
