import { describe, expect, it } from 'vitest';
import { isCodexCompatibleOpenAiModel } from '../hooks/useModelItems.hook';

describe('model item filtering', () => {
  it('matches the ChatGPT Codex-compatible OpenAI model policy', () => {
    expect(isCodexCompatibleOpenAiModel('gpt-5.5')).toBe(true);
    expect(isCodexCompatibleOpenAiModel('gpt-5.4')).toBe(true);
    expect(isCodexCompatibleOpenAiModel('gpt-5.6')).toBe(true);
    expect(isCodexCompatibleOpenAiModel('gpt-5.5-pro')).toBe(false);
    expect(isCodexCompatibleOpenAiModel('gpt-5.3')).toBe(false);
  });
});
