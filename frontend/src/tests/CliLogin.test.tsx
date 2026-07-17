import { fireEvent, render, waitFor } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { CliLogin } from '../pages/CliLogin';

const writeText = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  window.history.pushState({}, '', '/cli-login');
  Object.defineProperty(navigator, 'clipboard', {
    configurable: true,
    value: { writeText }
  });
});

describe('CliLogin', () => {
  it('renders and copies the exact one-time code without exchanging it', async () => {
    window.history.pushState({}, '', '/cli-login?code=one-time-code');
    writeText.mockResolvedValue(undefined);
    const fetchSpy = vi.spyOn(globalThis, 'fetch');

    const { getByText } = render(<CliLogin />);

    const code = getByText('one-time-code');
    expect(code.classList.contains('select-all')).toBe(true);
    fireEvent.click(getByText('Copy code'));

    await waitFor(() => expect(writeText).toHaveBeenCalledWith('one-time-code'));
    expect(getByText('Copied')).toBeDefined();
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('renders the explicit missing-code state without a copy action', () => {
    const { getByText, queryByText } = render(<CliLogin />);

    expect(
      getByText('Authorization code missing. Restart vulcanum login and complete GitHub sign-in.')
    ).toBeDefined();
    expect(queryByText('Copy code')).toBeNull();
  });

  it('keeps the code selectable when clipboard copying fails', async () => {
    window.history.pushState({}, '', '/cli-login?code=manual-code');
    writeText.mockRejectedValue(new Error('clipboard denied'));

    const { getByText } = render(<CliLogin />);
    fireEvent.click(getByText('Copy code'));

    await waitFor(() => expect(getByText('Copy failed. Select the code manually.')).toBeDefined());
    expect(getByText('manual-code').classList.contains('select-all')).toBe(true);
  });

  it('trims callback whitespace before rendering and copying', async () => {
    window.history.pushState({}, '', '/cli-login?code=%20trimmed-code%20');
    writeText.mockResolvedValue(undefined);

    const { getByText, queryByText } = render(<CliLogin />);
    fireEvent.click(getByText('Copy code'));

    await waitFor(() => expect(writeText).toHaveBeenCalledWith('trimmed-code'));
    expect(getByText('trimmed-code')).toBeDefined();
    expect(queryByText(' trimmed-code ')).toBeNull();
  });
});
