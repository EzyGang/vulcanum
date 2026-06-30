export type TeamAgentBackend = 'opencode' | 'omp_rpc';

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
  reviewEnabled: boolean;
  reviewMaxTurns: number;
  reviewPromptTemplate: string;
  maxInProgressTasks: number;
  agentBackend: TeamAgentBackend;
  createdAt: string;
}

export interface TeamMember {
  teamId: string;
  userId: string;
  email: string;
  role: 'owner' | 'member';
  createdAt: string;
}

export interface TeamDefaults {
  promptTemplate: string;
  reviewPromptTemplate: string;
  maxInProgressTasks: number;
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
  reviewEnabled?: boolean;
  reviewMaxTurns?: number;
  reviewPromptTemplate?: string;
  maxInProgressTasks?: number;
  agentBackend?: TeamAgentBackend;
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
