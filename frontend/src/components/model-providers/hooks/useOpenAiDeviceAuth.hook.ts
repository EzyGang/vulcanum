import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import {
  pollModelProviderDeviceFlow,
  startModelProviderDeviceFlow
} from '../../../services/model-providers/model-providers.service';
import type { StartDeviceFlowResponse } from '../../../types/model-providers';
import { useApiMutation } from '../../../utils/api/query/hooks';

type DeviceFlowStatus = 'idle' | 'pending' | 'connected';

interface StartOpenAiDeviceAuthInput {
  displayName?: string;
  onConnected: () => Promise<void>;
  onPollingError: (message: string) => void;
}

const sleep = (milliseconds: number): Promise<void> =>
  new Promise((resolve) => window.setTimeout(resolve, milliseconds));

export const useOpenAiDeviceAuth = () => {
  const startDeviceFlowMutation = useApiMutation(
    (input: Parameters<typeof startModelProviderDeviceFlow>[0]) =>
      startModelProviderDeviceFlow(input)
  );
  const pollDeviceFlowMutation = useApiMutation((attemptId: string) =>
    pollModelProviderDeviceFlow(attemptId)
  );

  const deviceFlow = useSignal<StartDeviceFlowResponse | null>(null);
  const deviceFlowStatus = useSignal<DeviceFlowStatus>('idle');
  const nextPollAt = useSignal<string | null>(null);

  const resetDeviceFlow = useCallback(() => {
    deviceFlow.value = null;
    deviceFlowStatus.value = 'idle';
    nextPollAt.value = null;
  }, []);

  const pollUntilConnected = useCallback(
    async (attemptId: string, intervalSeconds: number, onConnected: () => Promise<void>) => {
      let delayMs = intervalSeconds * 1000;
      while (deviceFlow.value?.attemptId === attemptId) {
        await sleep(delayMs);
        if (deviceFlow.value?.attemptId !== attemptId) return;
        const response = await pollDeviceFlowMutation.mutateAsync(attemptId);
        if (response.status === 'connected') {
          deviceFlowStatus.value = 'connected';
          await onConnected();
          return;
        }
        nextPollAt.value = response.nextPollAt;
        delayMs = Math.max(new Date(response.nextPollAt).getTime() - Date.now(), 1000);
      }
    },
    [pollDeviceFlowMutation]
  );

  const startOpenAiDeviceAuth = useCallback(
    async ({ displayName, onConnected, onPollingError }: StartOpenAiDeviceAuthInput) => {
      const flow = await startDeviceFlowMutation.mutateAsync({
        providerKey: 'openai',
        deviceProvider: 'openai_chatgpt',
        displayName
      });
      deviceFlow.value = flow;
      deviceFlowStatus.value = 'pending';
      nextPollAt.value = null;
      pollUntilConnected(flow.attemptId, flow.intervalSeconds, onConnected).catch((err) => {
        onPollingError(err instanceof Error ? err.message : 'Failed to poll device flow');
        deviceFlowStatus.value = 'idle';
      });
    },
    [startDeviceFlowMutation, pollUntilConnected]
  );

  return {
    data: { deviceFlow, deviceFlowStatus, nextPollAt },
    actions: { resetDeviceFlow, startOpenAiDeviceAuth }
  };
};
