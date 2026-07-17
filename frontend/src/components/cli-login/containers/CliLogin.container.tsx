import type { JSX } from 'preact';
import { useCliLogin } from '../hooks/useCliLogin.hook';
import { CliLoginView } from '../ui/CliLogin.view';

export const CliLoginContainer = (): JSX.Element => <CliLoginView {...useCliLogin()} />;
