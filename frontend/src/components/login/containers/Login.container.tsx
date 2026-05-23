import type { JSX } from 'preact';

import { useLogin } from '../hooks/useLogin.hook';
import { LoginView } from '../ui/Login.view';

export const LoginContainer = (): JSX.Element => {
  const { password, error, loading, handlePasswordChange, handleSubmit } = useLogin();

  return (
    <LoginView
      password={password}
      error={error}
      loading={loading}
      onPasswordChange={handlePasswordChange}
      onSubmit={handleSubmit}
    />
  );
};
