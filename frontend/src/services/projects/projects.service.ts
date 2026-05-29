import type {
  CreateProjectRequest,
  ProjectConfig,
  UpdateProjectRequest
} from '../../types/projects';
import { del, get, post, put } from '../../utils/api/request';

export const listProjects = (): Promise<ProjectConfig[]> => get<ProjectConfig[]>('/projects');

export const getProject = (id: string): Promise<ProjectConfig> =>
  get<ProjectConfig>(`/projects/${id}`);

export const createProject = (input: CreateProjectRequest): Promise<ProjectConfig> =>
  post<ProjectConfig>('/projects', input);

export const updateProject = (id: string, input: UpdateProjectRequest): Promise<ProjectConfig> =>
  put<ProjectConfig>(`/projects/${id}`, input);

export const deleteProject = (id: string): Promise<void> => del<void>(`/projects/${id}`);

interface ProjectsStats {
  enabledCount: number;
}

export const getProjectsStats = (): Promise<ProjectsStats> => get<ProjectsStats>('/projects/stats');
