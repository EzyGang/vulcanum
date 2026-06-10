import type { JSX } from 'preact';
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

  return (
    <LoginView
      data={{ password }}
      status={{ error, loading, modeLoading, isSingleUser }}
      actions={{
        onPasswordChange: handlePasswordChange,
        onSubmit: handleSubmit,
        onGithubLogin: handleGithubLogin
      }}
    />
  );
};
