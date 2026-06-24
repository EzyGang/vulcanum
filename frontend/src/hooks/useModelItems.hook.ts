import type { Signal } from '@preact/signals';
import type { CatalogProvider, ModelProviderConfig } from '../types/modelProviders';
import type { SelectOption } from '../types/shared';
import { modelProviderLabel } from '../utils/modelProviderAuth';

interface UseModelItemsParams {
  modelProviders: ModelProviderConfig[];
  catalogProviders: CatalogProvider[];
  primaryModelProviderConfigId: Signal<string>;
  smallModelProviderConfigId: Signal<string>;
}

export const useModelItems = ({
  modelProviders,
  catalogProviders,
  primaryModelProviderConfigId,
  smallModelProviderConfigId
}: UseModelItemsParams) => ({
  connectedProviderItems: modelProviders.map((provider) => ({
    value: provider.id,
    label: modelProviderLabel(provider)
  })),
  primaryModelItems: modelItemsForProvider(
    catalogProviders,
    modelProviders,
    primaryModelProviderConfigId.value
  ),
  smallModelItems: modelItemsForProvider(
    catalogProviders,
    modelProviders,
    smallModelProviderConfigId.value
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

export const modelProviderConfigIdForLegacyKey = (
  modelProviders: ModelProviderConfig[],
  providerConfigId?: string | null,
  providerKey?: string | null
): string => {
  if (providerConfigId) {
    return providerConfigId;
  }
  if (!providerKey) {
    return '';
  }
  return (
    modelProviders.find(
      (provider) => provider.providerKey === providerKey && provider.authType === 'api_key'
    )?.id ??
    modelProviders.find((provider) => provider.providerKey === providerKey)?.id ??
    ''
  );
};
