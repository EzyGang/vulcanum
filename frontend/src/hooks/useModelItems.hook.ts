import type { Signal } from '@preact/signals';
import type { CatalogProvider, ModelProviderConfig } from '../types/modelProviders';
import type { SelectOption } from '../types/shared';

interface UseModelItemsParams {
  modelProviders: ModelProviderConfig[];
  catalogProviders: CatalogProvider[];
  primaryModelProviderKey: Signal<string>;
  smallModelProviderKey: Signal<string>;
}

export const useModelItems = ({
  modelProviders,
  catalogProviders,
  primaryModelProviderKey,
  smallModelProviderKey
}: UseModelItemsParams) => ({
  connectedProviderItems: modelProviders.map((provider) => ({
    value: provider.id,
    label: providerLabel(provider)
  })),
  primaryModelItems: modelItemsForProvider(
    catalogProviders,
    modelProviders,
    primaryModelProviderKey.value
  ),
  smallModelItems: modelItemsForProvider(
    catalogProviders,
    modelProviders,
    smallModelProviderKey.value
  )
});

const modelItemsForProvider = (
  catalogProviders: CatalogProvider[],
  modelProviders: ModelProviderConfig[],
  providerConfigId: string
): SelectOption[] =>
  catalogProviders
    .find((provider) => provider.id === providerKeyForConfig(modelProviders, providerConfigId))
    ?.models.map((model) => ({ value: model.id, label: model.name })) ?? [];

const providerKeyForConfig = (modelProviders: ModelProviderConfig[], providerConfigId: string) =>
  modelProviders.find((provider) => provider.id === providerConfigId)?.providerKey ??
  providerConfigId;

const providerLabel = (provider: ModelProviderConfig): string => {
  const name = provider.displayName || provider.providerKey;
  const auth = provider.authType === 'chatgpt_oauth' ? 'ChatGPT Pro/Plus' : 'API Key';
  return `${name} (${auth})`;
};
