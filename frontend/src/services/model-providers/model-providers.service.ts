import type {
  CatalogResponse,
  CreateModelProviderRequest,
  ModelProviderConfig,
  UpdateModelProviderRequest
} from '../../types/modelProviders';
import { del, get, post, put } from '../../utils/api/request';

export const getModelProviderCatalog = (): Promise<CatalogResponse> =>
  get<CatalogResponse>('/model-providers/catalog');

export const listModelProviders = (): Promise<ModelProviderConfig[]> =>
  get<ModelProviderConfig[]>('/model-providers');

export const createModelProvider = (
  input: CreateModelProviderRequest
): Promise<ModelProviderConfig> => post<ModelProviderConfig>('/model-providers', input);

export const updateModelProvider = (
  id: string,
  input: UpdateModelProviderRequest
): Promise<ModelProviderConfig> => put<ModelProviderConfig>(`/model-providers/${id}`, input);

export const deleteModelProvider = (id: string): Promise<void> =>
  del<void>(`/model-providers/${id}`);
