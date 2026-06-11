import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../components/login/hooks/useLogin.hook', () => ({
  useLogin: vi.fn()
}));

import { LoginContainer } from '../components/login/containers/Login.container';
import { useLogin } from '../components/login/hooks/useLogin.hook';

describe('Login.container', () => {
  const password = signal('');
  const error = signal<string | null>(null);
  const loading = signal(false);
  let mode: 'loading' | 'single-user' | 'github' = 'single-user';
  const onPasswordChange = vi.fn();
  const onGithubLogin = vi.fn();
  const onSubmit = vi.fn(async (e: Event) => {
    e.preventDefault();
  });

  const getContent = () => {
    if (mode === 'loading') {
      return <div>Loading auth mode...</div>;
    }

    if (mode === 'github') {
      return (
        <button type='button' disabled={loading.value} onClick={onGithubLogin}>
          {loading.value ? 'Signing in...' : 'Sign in with GitHub'}
        </button>
      );
    }

    return (
      <form onSubmit={onSubmit}>
        <input
          type='password'
          value={password.value}
          onInput={onPasswordChange}
          placeholder='Instance password'
          disabled={loading.value}
        />
        {error.value && <div>{error.value}</div>}
        <button type='submit' disabled={loading.value}>
          {loading.value ? 'Signing in...' : 'Sign in'}
        </button>
      </form>
    );
  };

  const mockUseLogin = () => {
    vi.mocked(useLogin).mockReturnValue({
      data: {
        password
      },
      status: {
        error,
        loading
      },
      actions: {
        onPasswordChange,
        onSubmit,
        onGithubLogin
      },
      view: {
        content: getContent(),
        description:
          mode === 'github'
            ? 'Sign in with GitHub to create or access your team.'
            : 'Enter the instance password to continue.'
      }
    });
  };

  beforeEach(() => {
    password.value = '';
    error.value = null;
    loading.value = false;
    mode = 'single-user';
    vi.clearAllMocks();
    mockUseLogin();
  });

  const renderContainer = () => {
    mockUseLogin();
    return render(<LoginContainer />);
  };

  it('renders the password input and submit button', () => {
    const { getByPlaceholderText, getByText } = renderContainer();

    expect(getByPlaceholderText('Instance password')).toBeDefined();
    expect(getByText('Sign in')).toBeDefined();
    expect(getByText('Vulcanum')).toBeDefined();
  });

  it('calls onSubmit when the form is submitted', () => {
    const { getByText } = renderContainer();

    fireEvent.click(getByText('Sign in'));

    expect(onSubmit).toHaveBeenCalledOnce();
  });

  it('shows error message when error signal is set', () => {
    error.value = 'Invalid password';

    const { getByText } = renderContainer();

    expect(getByText('Invalid password')).toBeDefined();
  });

  it('disables submit button when loading', () => {
    loading.value = true;

    const { getByText } = renderContainer();

    const button = getByText('Signing in...') as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it('disables input when loading', () => {
    loading.value = true;

    const { getByPlaceholderText } = renderContainer();

    const input = getByPlaceholderText('Instance password') as HTMLInputElement;
    expect(input.disabled).toBe(true);
  });

  it('renders GitHub login in multi-user mode', () => {
    mode = 'github';
    mockUseLogin();

    const { getByText } = renderContainer();

    expect(getByText('Sign in with GitHub')).toBeDefined();
  });

  it('renders auth mode loading state', () => {
    mode = 'loading';
    mockUseLogin();

    const { getByText } = renderContainer();

    expect(getByText('Loading auth mode...')).toBeDefined();
  });
});
