import type {
  CreateProviderRequest,
  IntegrationProvider,
  LookupProjectResponse,
  ProjectInfo,
  UpdateProviderRequest,
  WorkspaceInfo
} from '../../types/projects';
import { del, get, patch, post } from '../../utils/api/request';

export const listProviders = (): Promise<IntegrationProvider[]> =>
  get<IntegrationProvider[]>('/providers');

export const getProvider = (id: string): Promise<IntegrationProvider> =>
  get<IntegrationProvider>(`/providers/${id}`);

export const createProvider = (input: CreateProviderRequest): Promise<IntegrationProvider> =>
  post<IntegrationProvider>('/providers', input);

export const updateProvider = (
  id: string,
  input: UpdateProviderRequest
): Promise<IntegrationProvider> => patch<IntegrationProvider>(`/providers/${id}`, input);

export const deleteProvider = (id: string): Promise<void> => del<void>(`/providers/${id}`);

export const lookupProject = async (
  providerId: string,
  externalProjectId: string
): Promise<LookupProjectResponse> =>
  get<LookupProjectResponse>(`/providers/${providerId}/projects/lookup`, {
    external_project_id: externalProjectId
  });

export const listWorkspaces = (providerId: string): Promise<WorkspaceInfo[]> =>
  get<WorkspaceInfo[]>(`/providers/${providerId}/workspaces`);

export const listProjects = (providerId: string, workspaceId: string): Promise<ProjectInfo[]> =>
  get<ProjectInfo[]>(`/providers/${providerId}/projects`, {
    workspace_id: workspaceId
  });
