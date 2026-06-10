import type { JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import { Input } from '../../shared/ui/Input.view';
import { useLogin } from '../hooks/useLogin.hook';
import { LoginView } from '../ui/Login.view';

export const LoginContainer = (): JSX.Element => {
  const {
    password,
    error,
    loading,
    modeLoading,
    isSingleUser,
    handlePasswordChange,
    handleSubmit,
    handleGithubLogin
  } = useLogin();

  const description = isSingleUser.value
    ? 'Enter the instance password to continue.'
    : 'Sign in with GitHub to create or access your team.';

  const content = modeLoading.value ? (
    <div class='text-text-muted text-sm'>Loading auth mode...</div>
  ) : isSingleUser.value ? (
    <form onSubmit={handleSubmit} class='flex flex-col gap-4'>
      <Input
        type='password'
        value={password.value}
        onInput={handlePasswordChange}
        placeholder='Instance password'
        autofocus
        disabled={loading.value}
      />

      {error.value && <div class='text-error text-sm'>{error.value}</div>}

      <Button type='submit' variant='primary' disabled={loading.value}>
        {loading.value ? 'Signing in...' : 'Sign in'}
      </Button>
    </form>
  ) : (
    <div class='flex flex-col gap-4'>
      {error.value && <div class='text-error text-sm'>{error.value}</div>}
      <Button type='button' variant='primary' disabled={loading.value} onClick={handleGithubLogin}>
        {loading.value ? 'Signing in...' : 'Sign in with GitHub'}
      </Button>
    </div>
  );

  return <LoginView description={description}>{content}</LoginView>;
};
