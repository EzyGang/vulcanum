import type {
  AcceptTeamInviteResponse,
  CreateTeamInviteResponse,
  CreateTeamRequest,
  Team,
  TeamInvitePreviewResponse,
  TeamMember,
  UpdateTeamRequest
} from '../../types/teams';
import { del, get, post, put } from '../../utils/api/request';

export const listTeams = () => get<Team[]>('/teams');

export const getTeam = (id: string) => get<Team>(`/teams/${id}`);

export const createTeam = (input: CreateTeamRequest) => post<Team>('/teams', input);

export const updateTeam = (id: string, input: UpdateTeamRequest) =>
  put<Team>(`/teams/${id}`, input);

export const deleteTeam = (id: string) => del<void>(`/teams/${id}`);

export const listTeamMembers = (teamId: string) => get<TeamMember[]>(`/teams/${teamId}/members`);

export const createTeamInvite = (teamId: string) =>
  post<CreateTeamInviteResponse>(`/teams/${teamId}/invites`);

export const previewTeamInvite = (token: string) =>
  get<TeamInvitePreviewResponse>(`/team-invites/${token}`);

export const acceptTeamInvite = (token: string) =>
  post<AcceptTeamInviteResponse>(`/team-invites/${token}/accept`);
