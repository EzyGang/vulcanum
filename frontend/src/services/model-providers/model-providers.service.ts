import type {
  CatalogResponse,
  CreateModelProviderRequest,
  ModelProviderConfig,
  PollDeviceFlowResponse,
  StartDeviceFlowRequest,
  StartDeviceFlowResponse,
  UpdateModelProviderRequest
} from '../../types/model-providers';
import { del, get, patch, post } from '../../utils/api/request';

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
): Promise<ModelProviderConfig> => patch<ModelProviderConfig>(`/model-providers/${id}`, input);

export const deleteModelProvider = (id: string): Promise<void> =>
  del<void>(`/model-providers/${id}`);

export const startModelProviderDeviceFlow = (
  input: StartDeviceFlowRequest
): Promise<StartDeviceFlowResponse> =>
  post<StartDeviceFlowResponse>('/model-providers/device-flows', input);

export const pollModelProviderDeviceFlow = (attemptId: string): Promise<PollDeviceFlowResponse> =>
  post<PollDeviceFlowResponse>(`/model-providers/device-flows/${attemptId}/poll`, {});
