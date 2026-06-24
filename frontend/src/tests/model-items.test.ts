import { signal } from '@preact/signals';
import { describe, expect, it } from 'vitest';
import { useModelItems } from '../hooks/useModelItems.hook';

describe('model item filtering', () => {
  it('uses server-provided ChatGPT model compatibility for OAuth-backed OpenAI providers', () => {
    const result = useModelItems({
      modelProviders: [
        {
          id: 'provider-id',
          teamId: 'team-id',
          providerKey: 'openai',
          displayName: 'ChatGPT',
          authType: 'device_oauth',
          credentialFields: [],
          createdAt: '2026-01-01T00:00:00Z',
          updatedAt: '2026-01-01T00:00:00Z'
        }
      ],
      catalogProviders: [
        {
          id: 'openai',
          name: 'OpenAI',
          doc: '',
          env: [],
          models: [model('gpt-5.5', 'GPT 5.5', true), model('gpt-5.5-pro', 'GPT 5.5 Pro', false)]
        }
      ],
      primaryModelProviderKey: signal('openai'),
      smallModelProviderKey: signal('openai')
    });

    expect(result.primaryModelItems).toEqual([{ value: 'gpt-5.5', label: 'GPT 5.5' }]);
  });
});

const model = (id: string, name: string, opencodeChatgptCompatible: boolean) => ({
  id,
  name,
  attachment: false,
  reasoning: true,
  toolCall: true,
  structuredOutput: true,
  opencodeChatgptCompatible
});
