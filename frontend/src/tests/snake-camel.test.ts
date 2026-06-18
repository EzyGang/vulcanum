import { describe, expect, it } from 'vitest';
import { camelKeys, snakeKeys } from '../utils/api/snake-camel';

describe('api case conversion', () => {
  it('preserves model provider credential keys when snake-casing requests', () => {
    const result = snakeKeys({
      providerKey: 'deepseek',
      credentials: {
        DEEPSEEK_API_KEY: 'secret'
      }
    });

    expect(result).toEqual({
      provider_key: 'deepseek',
      credentials: {
        DEEPSEEK_API_KEY: 'secret'
      }
    });
  });

  it('preserves model provider credential keys when camel-casing responses', () => {
    const result = camelKeys({
      provider_key: 'deepseek',
      credentials: {
        DEEPSEEK_API_KEY: 'secret'
      }
    });

    expect(result).toEqual({
      providerKey: 'deepseek',
      credentials: {
        DEEPSEEK_API_KEY: 'secret'
      }
    });
  });
});
