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
  providerKey: string;
  authType: 'api_key' | 'chatgpt_oauth';
  displayName: string;
  credentials: Record<string, string>;
  oauthMetadata?: {
    accountId?: string | null;
    email?: string | null;
    expiresAt?: string | null;
  };
  createdAt: string;
  updatedAt: string;
}

export interface CreateModelProviderRequest {
  providerKey: string;
  authType?: 'api_key' | 'chatgpt_oauth';
  displayName?: string;
  credentials: Record<string, string>;
}

export interface UpdateModelProviderRequest {
  displayName?: string;
  credentials?: Record<string, string>;
}

export interface StartChatGptAuthRequest {
  displayName?: string;
}

export interface ChatGptAuthStartResponse {
  attemptId: string;
  verificationUri: string;
  userCode: string;
  expiresAt: string;
  pollIntervalSeconds: number;
}

export interface ChatGptAuthStatusResponse {
  status: 'pending' | 'complete' | 'expired' | 'failed';
  error?: string | null;
  provider?: ModelProviderConfig | null;
}
