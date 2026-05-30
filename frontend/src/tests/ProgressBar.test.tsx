import { render } from '@testing-library/preact';
import { describe, expect, it } from 'vitest';

import { ProgressBar } from '../components/shared/ui/ProgressBar.view';

describe('ProgressBar.view', () => {
  it('renders the fraction text when showFraction is true', () => {
    const { getByText } = render(<ProgressBar value={2} max={5} showFraction />);

    expect(getByText('2 / 5')).toBeDefined();
  });

  it('does not render fraction text when showFraction is omitted', () => {
    const { container } = render(<ProgressBar value={2} max={5} />);

    expect(container.textContent).not.toContain('2 / 5');
  });

  it('uses bg-error when value reaches max', () => {
    const { container } = render(<ProgressBar value={5} max={5} />);

    expect(container.querySelector('.bg-error')).toBeDefined();
  });

  it('clamps value to max and renders bg-error when value exceeds max', () => {
    const { container } = render(<ProgressBar value={8} max={5} />);

    expect(container.querySelector('.bg-error')).toBeDefined();
  });

  it('uses bg-success when ratio is below 50%', () => {
    const { container } = render(<ProgressBar value={1} max={3} />);

    expect(container.querySelector('.bg-success')).toBeDefined();
    expect(container.querySelector('.bg-warning')).toBeNull();
    expect(container.querySelector('.bg-error')).toBeNull();
  });

  it('uses bg-warning when ratio is 50% or above but below 100%', () => {
    const { container: at50 } = render(<ProgressBar value={2} max={4} />);
    expect(at50.querySelector('.bg-warning')).toBeDefined();

    const { container: at99 } = render(<ProgressBar value={99} max={100} />);
    expect(at99.querySelector('.bg-warning')).toBeDefined();
  });

  it('uses bg-error when ratio is 100%', () => {
    const { container } = render(<ProgressBar value={3} max={3} />);

    expect(container.querySelector('.bg-error')).toBeDefined();
    expect(container.querySelector('.bg-warning')).toBeNull();
    expect(container.querySelector('.bg-success')).toBeNull();
  });
});
