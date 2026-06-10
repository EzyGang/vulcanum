import type {
  AuthModeResponse,
  InstanceLoginRequest,
  InstanceLoginResponse,
  MeResponse
} from '../../types/auth';
import { get, post } from '../../utils/api/request';

export const instanceLogin = (password: string) =>
  post<InstanceLoginResponse>('/auth/instance-login', {
    password
  } as InstanceLoginRequest);

export const getAuthMode = () => get<AuthModeResponse>('/auth/mode');

export const getMe = () => get<MeResponse>('/auth/me');

export const getGithubLoginUrl = () => '/api/v1/auth/github/start';
