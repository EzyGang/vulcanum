export interface InstanceLoginRequest {
  password: string;
}

export interface AuthTokenResponse {
  accessToken: string;
  refreshToken: string;
  refreshExpiresAt: string;
}

export interface RefreshRequest {
  refreshToken: string;
}

export interface AuthExchangeRequest {
  code: string;
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

export interface AuthIdentity {
  provider: string;
  providerUserId: string;
  login: string;
  verifiedAt: string | null;
}

export interface MeResponse {
  user: AuthUser;
  teams: AuthTeam[];
  identities: AuthIdentity[];
}
