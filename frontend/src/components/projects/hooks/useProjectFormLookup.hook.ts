import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import { lookupProject } from '../../../services/providers/providers.service';
import type { ColumnInfo } from '../../../types/projects';

export const useProjectFormLookup = (
  providerId: { readonly value: string },
  externalProjectId: { readonly value: string }
) => {
  const columns = useSignal<ColumnInfo[]>([]);
  const columnsLoading = useSignal(false);
  const lookupProjectName = useSignal('');
  const lookupError = useSignal<string | null>(null);
  const lookedUp = useSignal(false);

  const handleLookup = useCallback(async () => {
    if (!providerId.value || !externalProjectId.value) return;

    lookupError.value = null;
    columnsLoading.value = true;
    lookedUp.value = false;

    try {
      const result = await lookupProject(providerId.value, externalProjectId.value);
      lookupProjectName.value = result.name;
      columns.value = result.columns;
      lookedUp.value = true;
    } catch (err) {
      lookupError.value = err instanceof Error ? err.message : 'Lookup failed';
      columns.value = [];
      lookupProjectName.value = '';
    } finally {
      columnsLoading.value = false;
    }
  }, []);

  const resetLookup = useCallback(() => {
    lookedUp.value = false;
    columns.value = [];
    lookupProjectName.value = '';
    lookupError.value = null;
  }, []);

  return {
    columns,
    columnsLoading,
    lookupProjectName,
    lookupError,
    lookedUp,
    handleLookup,
    resetLookup
  };
};
