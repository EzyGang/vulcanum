import type { ModelProviderConfig } from '../types/modelProviders';

export const modelProviderAuthLabel = (authType: ModelProviderConfig['authType']): string =>
  authType === 'chatgpt_oauth' ? 'ChatGPT Pro/Plus' : 'API Key';

export const modelProviderLabel = (provider: ModelProviderConfig): string => {
  const name = provider.displayName || provider.providerKey;
  return `${name} (${modelProviderAuthLabel(provider.authType)})`;
};
