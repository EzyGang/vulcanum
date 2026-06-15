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
    value: provider.providerKey,
    label: provider.displayName || provider.providerKey
  })),
  primaryModelItems: modelItemsForProvider(catalogProviders, primaryModelProviderKey.value),
  smallModelItems: modelItemsForProvider(catalogProviders, smallModelProviderKey.value)
});

const modelItemsForProvider = (
  catalogProviders: CatalogProvider[],
  providerKey: string
): SelectOption[] =>
  catalogProviders
    .find((provider) => provider.id === providerKey)
    ?.models.map((model) => ({ value: model.id, label: model.name })) ?? [];
