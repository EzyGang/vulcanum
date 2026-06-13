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
  displayName: string;
  credentials: Record<string, string>;
  advancedOptions: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export interface CreateModelProviderRequest {
  providerKey: string;
  displayName?: string;
  credentials: Record<string, string>;
  advancedOptions?: Record<string, unknown>;
}

export interface UpdateModelProviderRequest {
  displayName?: string;
  credentials?: Record<string, string>;
  advancedOptions?: Record<string, unknown>;
}
