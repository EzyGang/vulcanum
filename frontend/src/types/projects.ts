export interface ProjectConfig {
  id: string;
  externalProjectId: string;
  name: string;
  externalWorkspaceId: string;
  enabled: boolean;
  pickupColumn: string;
  targetColumn: string;
  progressColumn: string;
  promptTemplate: string;
  repoUrl: string;
  agentsMd: string;
  opencodeConfig: string;
  primaryModelProviderKey?: string;
  primaryModelId?: string;
  smallModelProviderKey?: string;
  smallModelId?: string;
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
  targetColumn?: string;
  promptTemplate: string;
  repoUrl?: string;
  agentsMd?: string;
  opencodeConfig?: string;
  primaryModelProviderKey?: string;
  primaryModelId?: string;
  smallModelProviderKey?: string;
  smallModelId?: string;
}

export interface UpdateProjectRequest {
  enabled?: boolean;
  externalWorkspaceId?: string;
  name?: string;
  pickupColumn?: string;
  progressColumn?: string;
  targetColumn?: string;
  promptTemplate?: string;
  repoUrl?: string;
  agentsMd?: string;
  opencodeConfig?: string;
  primaryModelProviderKey?: string;
  primaryModelId?: string;
  smallModelProviderKey?: string;
  smallModelId?: string;
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
