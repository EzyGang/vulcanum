import type {
  AuthModeResponse,
  AuthTokenResponse,
  InstanceLoginRequest,
  InstanceLoginResponse,
  MeResponse,
  RefreshRequest
} from '../../types/auth';
import { get, post } from '../../utils/api/request';

export const instanceLogin = (password: string) =>
  post<InstanceLoginResponse>('/auth/instance-login', {
    password
  } as InstanceLoginRequest);

export const getAuthMode = () => get<AuthModeResponse>('/auth/mode');

export const getMe = () => get<MeResponse>('/auth/me');

export const refreshAuth = (refreshToken: string) =>
  post<AuthTokenResponse>('/auth/refresh', {
    refreshToken
  } as RefreshRequest);

export const getGithubLoginUrl = () => '/api/v1/auth/github/start';
