import type {
  CatalogResponse,
  ChatGptAuthStartResponse,
  ChatGptAuthStatusResponse,
  CreateModelProviderRequest,
  ModelProviderConfig,
  StartChatGptAuthRequest,
  UpdateModelProviderRequest
} from '../../types/modelProviders';
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

export const startChatGptAuth = (
  input: StartChatGptAuthRequest
): Promise<ChatGptAuthStartResponse> =>
  post<ChatGptAuthStartResponse>('/model-providers/openai-chatgpt/auth/start', input);

export const getChatGptAuthStatus = (attemptId: string): Promise<ChatGptAuthStatusResponse> =>
  get<ChatGptAuthStatusResponse>(`/model-providers/openai-chatgpt/auth/${attemptId}`);

export const cancelChatGptAuth = (attemptId: string): Promise<void> =>
  post<void>(`/model-providers/openai-chatgpt/auth/${attemptId}/cancel`, {});
