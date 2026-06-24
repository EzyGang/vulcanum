import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import {
  getModelProviderCatalog,
  listModelProviders
} from '../services/model-providers/model-providers.service';
import { useApiQuery } from '../utils/api/query/hooks';
import {
  modelProviderConfigIdForLegacyKey as resolveModelProviderConfigIdForLegacyKey,
  useModelItems
} from './useModelItems.hook';

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
  const onPrimaryProviderChange = useCallback((value: string) => {
    primaryModelProviderKey.value = value;
    primaryModelId.value = '';
  }, []);
  const onPrimaryModelChange = useCallback((value: string) => {
    primaryModelId.value = value;
  }, []);
  const onSmallProviderChange = useCallback((value: string) => {
    smallModelProviderKey.value = value;
    smallModelId.value = '';
  }, []);
  const onSmallModelChange = useCallback((value: string) => {
    smallModelId.value = value;
  }, []);
  const modelProviderConfigIdForLegacyKey = useCallback(
    (providerConfigId?: string | null, providerKey?: string | null) =>
      resolveModelProviderConfigIdForLegacyKey(modelProviders, providerConfigId, providerKey),
    [modelProviders]
  );

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
    onPrimaryProviderChange,
    onPrimaryModelChange,
    onSmallProviderChange,
    onSmallModelChange,
    modelProviderConfigIdForLegacyKey,
    needsLegacyModelProviderResolution
  };
};

const needsLegacyModelProviderResolution = (selection: LegacyModelProviderSelection): boolean =>
  (!selection.primaryModelProviderConfigId && !!selection.primaryModelProviderKey) ||
  (!selection.smallModelProviderConfigId && !!selection.smallModelProviderKey);
