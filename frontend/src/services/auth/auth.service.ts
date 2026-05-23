import type { InstanceLoginRequest, InstanceLoginResponse } from '../../types/auth';
import { post } from '../../utils/api/request';

export const instanceLogin = (password: string) =>
  post<InstanceLoginResponse>('/auth/instance-login', {
    password
  } as InstanceLoginRequest);
