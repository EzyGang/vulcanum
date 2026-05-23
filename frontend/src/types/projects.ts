export interface ProjectConfig {
  id: string;
  kaneoProjectId: string;
  enabled: boolean;
  pickupColumn: string;
  targetColumn: string;
  progressColumn: string;
  promptTemplate: string;
  repoUrl: string;
  agentsMd: string;
  createdAt: string;
}

export interface CreateProjectRequest {
  kaneoProjectId: string;
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
}

export interface ColumnInfo {
  id: string;
  name: string;
}

export interface ColumnsResponse {
  columns: ColumnInfo[];
}
