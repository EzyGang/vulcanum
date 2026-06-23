import { useSignal } from '@preact/signals';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../services/model-providers/model-providers.service';
import { useApiQuery } from '../utils/api/query/hooks';
import { modelProviderConfigIdForLegacyKey, useModelItems } from './useModelItems.hook';

interface LegacyModelProviderSelection {
  primaryModelProviderConfigId?: string | null;
  primaryModelProviderKey?: string | null;
  smallModelProviderConfigId?: string | null;
  smallModelProviderKey?: string | null;
}

export const useModelProviderSelection = () => {
  const primaryModelProviderKey = useSignal('');
  const primaryModelId = useSignal('');
  const smallModelProviderKey = useSignal('');
  const smallModelId = useSignal('');
  const { data: modelProviders = [], isLoading: modelProvidersLoading } = useApiQuery(
    ['model-providers'],
    () => listModelProviders()
  );
  const { data: modelCatalog } = useApiQuery(['model-provider-catalog'], () =>
    getModelProviderCatalog()
  );
  const catalogProviders = modelCatalog?.providers ?? [];
  const { connectedProviderItems, primaryModelItems, smallModelItems } = useModelItems({
    modelProviders,
    catalogProviders,
    primaryModelProviderKey,
    smallModelProviderKey
  });

  return {
    modelProviders,
    modelProvidersLoading,
    catalogProviders,
    primaryModelProviderKey,
    primaryModelId,
    smallModelProviderKey,
    smallModelId,
    connectedProviderItems,
    primaryModelItems,
    smallModelItems,
    onPrimaryProviderChange: (value: string) => {
      primaryModelProviderKey.value = value;
      primaryModelId.value = '';
    },
    onPrimaryModelChange: (value: string) => {
      primaryModelId.value = value;
    },
    onSmallProviderChange: (value: string) => {
      smallModelProviderKey.value = value;
      smallModelId.value = '';
    },
    onSmallModelChange: (value: string) => {
      smallModelId.value = value;
    },
    modelProviderConfigIdForLegacyKey: (
      providerConfigId?: string | null,
      providerKey?: string | null
    ) => modelProviderConfigIdForLegacyKey(modelProviders, providerConfigId, providerKey),
    needsLegacyModelProviderResolution
  };
};

const needsLegacyModelProviderResolution = (selection: LegacyModelProviderSelection): boolean =>
  (!selection.primaryModelProviderConfigId && !!selection.primaryModelProviderKey) ||
  (!selection.smallModelProviderConfigId && !!selection.smallModelProviderKey);
