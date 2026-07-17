import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';

type CopyStatus = 'idle' | 'copied' | 'failed';
export type CliLoginMode = 'code' | 'missing';

export interface CliLoginViewProps {
  data: {
    code: string;
  };
  status: {
    copyMessage: string;
  };
  actions: {
    onCopy: () => void;
  };
  view: {
    mode: CliLoginMode;
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
    status: { copyMessage: getCopyMessage(copy.value) },
    actions: { onCopy: handleCopy },
    view: { mode: code ? 'code' : 'missing' }
  };
};

const getCopyMessage = (status: CopyStatus): string => {
  switch (status) {
    case 'copied':
      return 'Copied';
    case 'failed':
      return 'Copy failed. Select the code manually.';
    case 'idle':
      return '';
  }
};
