import type { JSX } from 'preact';
import { useCliLogin } from '../hooks/useCliLogin.hook';
import { CliLoginCodeView, CliLoginMissingView } from '../ui/CliLogin.view';

export const CliLoginContainer = (): JSX.Element => {
  const login = useCliLogin();
  return login.view.mode === 'code' ? <CliLoginCodeView {...login} /> : <CliLoginMissingView />;
};
