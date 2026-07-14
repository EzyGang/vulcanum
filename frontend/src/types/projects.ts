export interface ProjectConfig {
  id: string;
  externalProjectId: string;
  name: string;
  externalWorkspaceId: string;
  enabled: boolean;
  pickupColumn: string;
  reviewColumn: string;
  doneColumn: string;
  progressColumn: string;
  promptTemplate?: string | null;
  repoFullNames?: string[];
  repoUrls?: string[];
  agentsMd?: string | null;
  reviewEnabled?: boolean | null;
  reviewMaxTurns?: number | null;
  reviewPromptTemplate?: string | null;
  maxInProgressTasks?: number | null;
  createdAt: string;
  providerId?: string;
}

export interface CreateProjectRequest {
  externalProjectId: string;
  externalWorkspaceId?: string;
  name?: string;
  providerId: string;
  enabled?: boolean;
  pickupColumn?: string;
  progressColumn?: string;
  reviewColumn?: string;
  doneColumn?: string;
  promptTemplate?: string | null;
  repoFullNames?: string[];
  agentsMd?: string | null;
  reviewEnabled?: boolean | null;
  reviewMaxTurns?: number | null;
  reviewPromptTemplate?: string | null;
  maxInProgressTasks?: number | null;
}

export interface UpdateProjectRequest {
  enabled?: boolean;
  externalWorkspaceId?: string;
  name?: string;
  pickupColumn?: string;
  progressColumn?: string;
  reviewColumn?: string;
  doneColumn?: string;
  promptTemplate?: string | null;
  repoFullNames?: string[];
  agentsMd?: string | null;
  reviewEnabled?: boolean | null;
  reviewMaxTurns?: number | null;
  reviewPromptTemplate?: string | null;
  maxInProgressTasks?: number | null;
  providerId?: string;
}

export interface ColumnInfo {
  id: string;
  name: string;
  slug: string;
}

export interface ColumnsResponse {
  columns: ColumnInfo[];
}

export interface IntegrationProvider {
  id: string;
  name: string;
  providerType: string;
  instanceUrl: string;
  apiKey: string;
  createdAt: string;
}

export interface CreateProviderRequest {
  name: string;
  providerType?: string;
  instanceUrl: string;
  apiKey: string;
}

export interface UpdateProviderRequest {
  name?: string;
  providerType?: string;
  instanceUrl?: string;
  apiKey?: string;
}

export interface LookupProjectResponse {
  id: string;
  name: string;
  slug: string;
  columns: ColumnInfo[];
}

export interface WorkspaceInfo {
  id: string;
  name: string;
}

export interface ProjectInfo {
  id: string;
  name: string;
  slug: string;
}
