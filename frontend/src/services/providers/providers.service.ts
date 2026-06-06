import type {
  CreateProviderRequest,
  IntegrationProvider,
  LookupProjectResponse,
  UpdateProviderRequest
} from '../../types/projects';
import { del, get, post, put } from '../../utils/api/request';

export const listProviders = (): Promise<IntegrationProvider[]> =>
  get<IntegrationProvider[]>('/providers');

export const getProvider = (id: string): Promise<IntegrationProvider> =>
  get<IntegrationProvider>(`/providers/${id}`);

export const createProvider = (input: CreateProviderRequest): Promise<IntegrationProvider> =>
  post<IntegrationProvider>('/providers', input);

export const updateProvider = (
  id: string,
  input: UpdateProviderRequest
): Promise<IntegrationProvider> => put<IntegrationProvider>(`/providers/${id}`, input);

export const deleteProvider = (id: string): Promise<void> => del<void>(`/providers/${id}`);

export const lookupProject = async (
  providerId: string,
  externalProjectId: string
): Promise<LookupProjectResponse> =>
  get<LookupProjectResponse>(`/providers/${providerId}/projects/lookup`, {
    external_project_id: externalProjectId
  });
