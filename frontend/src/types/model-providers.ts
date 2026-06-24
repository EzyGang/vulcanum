export interface CatalogModel {
  id: string;
  name: string;
  status?: string;
  contextLimit?: number;
  outputLimit?: number;
  inputCost?: number;
  outputCost?: number;
  attachment: boolean;
  reasoning: boolean;
  toolCall: boolean;
  structuredOutput: boolean;
  opencodeChatgptCompatible: boolean;
}

export interface CatalogProvider {
  id: string;
  name: string;
  doc: string;
  env: string[];
  models: CatalogModel[];
}

export interface CatalogResponse {
  providers: CatalogProvider[];
}

export interface ModelProviderConfig {
  id: string;
  teamId: string;
  providerKey: string;
  displayName: string;
  authType: ModelProviderAuthType;
  credentialFields: string[];
  oauth?: ModelProviderOAuthStatus | null;
  createdAt: string;
  updatedAt: string;
}

export type ModelProviderAuthType = 'none' | 'api_key' | 'device_oauth';

export interface ModelProviderOAuthStatus {
  provider: string;
  accountId?: string | null;
  email?: string | null;
  expires?: number | null;
}

export interface CreateModelProviderRequest {
  providerKey: string;
  displayName?: string;
  authType: 'api_key';
  credentials: Record<string, string>;
}

export interface UpdateModelProviderRequest {
  displayName?: string;
  authType?: 'api_key';
  credentials?: Record<string, string>;
}

export interface StartDeviceFlowRequest {
  providerKey: 'openai';
  deviceProvider: 'openai_chatgpt';
  displayName?: string;
}

export interface StartDeviceFlowResponse {
  attemptId: string;
  verificationUri: string;
  userCode: string;
  intervalSeconds: number;
  expiresAt: string;
}

export type PollDeviceFlowResponse =
  | { status: 'pending'; nextPollAt: string }
  | { status: 'connected'; provider: ModelProviderConfig };
