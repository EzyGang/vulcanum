export interface Team {
  id: string;
  name: string;
  personalUserId: string | null;
  promptTemplate: string;
  agentsMd: string;
  primaryModelProviderKey?: string | null;
  primaryModelId?: string | null;
  smallModelProviderKey?: string | null;
  smallModelId?: string | null;
  createdAt: string;
}

export interface TeamMember {
  teamId: string;
  userId: string;
  email: string;
  role: 'owner' | 'member';
  createdAt: string;
}

export interface CreateTeamRequest {
  name: string;
}

export interface UpdateTeamRequest {
  name?: string;
  promptTemplate?: string;
  agentsMd?: string;
  primaryModelProviderKey?: string | null;
  primaryModelId?: string | null;
  smallModelProviderKey?: string | null;
  smallModelId?: string | null;
}

export interface CreateTeamInviteResponse {
  token: string;
  expiresAt: string;
}

export interface TeamInvitePreviewResponse {
  expiresAt: string;
}

export interface AcceptTeamInviteResponse {
  teamId: string;
}
