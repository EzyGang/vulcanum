import { type Signal, useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';

export type CopyStatus = 'idle' | 'copied' | 'failed';

export interface CliLoginViewProps {
  data: {
    code: string;
  };
  status: {
    copy: Signal<CopyStatus>;
  };
  actions: {
    onCopy: () => void;
  };
}

export const useCliLogin = (): CliLoginViewProps => {
  const code = new URLSearchParams(window.location.search).get('code')?.trim() ?? '';
  const copy = useSignal<CopyStatus>('idle');

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      copy.value = 'copied';
    } catch {
      copy.value = 'failed';
    }
  }, [code]);

  return {
    data: { code },
    status: { copy },
    actions: { onCopy: handleCopy }
  };
};
