import { render } from '@testing-library/preact';
import { describe, expect, it } from 'vitest';
import { RunBlockedReason } from '../components/runs/ui/runs-table/RunBlockedReason.view';

describe('RunBlockedReason', () => {
  it('renders the backend-provided reason', () => {
    const { getByText } = render(<RunBlockedReason reason='Waiting for repository access.' />);

    expect(getByText('Waiting for repository access.')).toBeDefined();
  });

  it('does not render a fallback when the backend provides no reason', () => {
    const { container } = render(<RunBlockedReason reason={null} />);

    expect(container.textContent).toBe('');
  });
});
