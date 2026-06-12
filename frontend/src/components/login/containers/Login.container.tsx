import type { JSX } from 'preact';
import { useLogin } from '../hooks/useLogin.hook';
import { LoginView } from '../ui/Login.view';

export const LoginContainer = (): JSX.Element => <LoginView {...useLogin()} />;
