import type { Signal } from '@preact/signals';
import type { CatalogProvider, ModelProviderConfig } from '../types/model-providers';
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
    value: provider.providerKey,
    label: provider.displayName || provider.providerKey
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
  providerKey: string
): SelectOption[] =>
  catalogProviders
    .find((provider) => provider.id === providerKey)
    ?.models.filter((model) => shouldShowModel(modelProviders, providerKey, model.id))
    .map((model) => ({ value: model.id, label: model.name })) ?? [];

const shouldShowModel = (
  modelProviders: ModelProviderConfig[],
  providerKey: string,
  modelId: string
): boolean => {
  const connectedProvider = modelProviders.find((provider) => provider.providerKey === providerKey);
  if (providerKey !== 'openai' || connectedProvider?.authType !== 'device_oauth') {
    return true;
  }
  return isCodexCompatibleOpenAiModel(modelId);
};

export const isCodexCompatibleOpenAiModel = (modelId: string): boolean => {
  if (modelId === 'gpt-5.5-pro') return false;
  if (['gpt-5.5', 'gpt-5.3-codex-spark', 'gpt-5.4', 'gpt-5.4-mini'].includes(modelId)) {
    return true;
  }

  const match = /^gpt-(\d+)\.(\d+)$/.exec(modelId);
  if (!match) return false;
  const major = Number(match[1]);
  const minor = Number(match[2]);
  return major > 5 || (major === 5 && minor > 4);
};
