import { describe, expect, it } from 'vitest';
import { RUN_USAGE_STATS } from '../components/runs/ui/runs-table/RunUsageStats';

describe('RUN_USAGE_STATS', () => {
  it('uses directional icons that match token flow', () => {
    expect(RUN_USAGE_STATS).toEqual([
      { field: 'inputTokens', icon: '↑', label: 'Input tokens' },
      { field: 'outputTokens', icon: '↓', label: 'Output tokens' },
      { field: 'cacheReadTokens', icon: '↙', label: 'Cache read tokens' },
      { field: 'cacheWriteTokens', icon: '↗', label: 'Cache write tokens' }
    ]);
  });
});
