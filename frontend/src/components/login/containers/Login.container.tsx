import type { JSX } from 'preact';
import { useLogin } from '../hooks/useLogin.hook';
import { LoginView } from '../ui/Login.view';

export const LoginContainer = (): JSX.Element => {
  const { data, status, actions, view } = useLogin();

  return <LoginView data={data} status={status} actions={actions} view={view} />;
};
