export interface Team {
  id: string;
  name: string;
  personalUserId: string | null;
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
  name: string;
}
