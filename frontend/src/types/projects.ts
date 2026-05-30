export interface ProjectConfig {
  id: string;
  kaneoProjectId: string;
  kaneoWorkspaceId: string;
  enabled: boolean;
  pickupColumn: string;
  targetColumn: string;
  progressColumn: string;
  promptTemplate: string;
  repoUrl: string;
  agentsMd: string;
  createdAt: string;
  providerId?: string;
}

export interface CreateProjectRequest {
  kaneoProjectId: string;
  providerId: string;
  enabled?: boolean;
  pickupColumn?: string;
  progressColumn?: string;
  targetColumn?: string;
  promptTemplate: string;
  repoUrl?: string;
  agentsMd?: string;
}

export interface UpdateProjectRequest {
  enabled?: boolean;
  pickupColumn?: string;
  progressColumn?: string;
  targetColumn?: string;
  promptTemplate?: string;
  repoUrl?: string;
  agentsMd?: string;
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
  name: string;
  columns: ColumnInfo[];
}
