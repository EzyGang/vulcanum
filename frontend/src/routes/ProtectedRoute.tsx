import type { ComponentChildren, JSX } from 'preact';
import { useLocation } from 'wouter-preact';

import { accessToken } from '../stores/auth.store';

interface ProtectedRouteProps {
  children: ComponentChildren;
}

export const ProtectedRoute = ({ children }: ProtectedRouteProps): JSX.Element | null => {
  const [_, setLocation] = useLocation();

  if (!accessToken.value) {
    setLocation('/login');
    return null;
  }

  return <>{children}</>;
};
