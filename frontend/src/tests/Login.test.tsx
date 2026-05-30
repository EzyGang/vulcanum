import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { LoginView } from '../components/login/ui/Login.view';

describe('Login.view', () => {
  const password = signal('');
  const error = signal<string | null>(null);
  const loading = signal(false);
  const onPasswordChange = vi.fn();
  const onSubmit = vi.fn((e: Event) => {
    e.preventDefault();
  });

  beforeEach(() => {
    password.value = '';
    error.value = null;
    loading.value = false;
    vi.clearAllMocks();
  });

  const renderView = () =>
    render(
      <LoginView
        data={{ password }}
        status={{ error, loading }}
        actions={{ onPasswordChange, onSubmit }}
      />
    );

  it('renders the password input and submit button', () => {
    const { getByPlaceholderText, getByText } = renderView();

    expect(getByPlaceholderText('Instance password')).toBeDefined();
    expect(getByText('Sign in')).toBeDefined();
    expect(getByText('Vulcanum')).toBeDefined();
  });

  it('calls onSubmit when the form is submitted', () => {
    const { getByText } = renderView();

    fireEvent.click(getByText('Sign in'));

    expect(onSubmit).toHaveBeenCalledOnce();
  });

  it('shows error message when error signal is set', () => {
    error.value = 'Invalid password';

    const { getByText } = renderView();

    expect(getByText('Invalid password')).toBeDefined();
  });

  it('disables submit button when loading', () => {
    loading.value = true;

    const { getByText } = renderView();

    const button = getByText('Signing in...') as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it('disables input when loading', () => {
    loading.value = true;

    const { getByPlaceholderText } = renderView();

    const input = getByPlaceholderText('Instance password') as HTMLInputElement;
    expect(input.disabled).toBe(true);
  });
});
