export interface InstanceLoginRequest {
  password: string;
}

export interface InstanceLoginResponse {
  token: string;
}

export interface AuthModeResponse {
  isSingleUser: boolean;
}

export interface AuthUser {
  id: string;
  email: string;
}

export interface AuthTeam {
  id: string;
  name: string;
}

export interface MeResponse {
  user: AuthUser;
  teams: AuthTeam[];
}
