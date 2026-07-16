import type {
  AuthExchangeRequest,
  AuthModeResponse,
  AuthTokenResponse,
  InstanceLoginRequest,
  MeResponse,
  RefreshRequest
} from '../../types/auth';
import { get, post } from '../../utils/api/request';

export const instanceLogin = (password: string) =>
  post<AuthTokenResponse>('/auth/instance-login', {
    password
  } as InstanceLoginRequest);

export const getAuthMode = () => get<AuthModeResponse>('/auth/mode');

export const getMe = () => get<MeResponse>('/auth/me');

export const exchangeAuthCode = (code: string) =>
  post<AuthTokenResponse>('/auth/exchange', {
    code
  } as AuthExchangeRequest);

export const refreshAuth = (refreshToken: string) =>
  post<AuthTokenResponse>('/auth/refresh', {
    refreshToken
  } as RefreshRequest);

export const getGithubLoginUrl = (returnTo?: string) => {
  if (!returnTo) return '/api/v1/auth/github/start';

  const params = new URLSearchParams({ return_to: returnTo });
  return `/api/v1/auth/github/start?${params.toString()}`;
};
