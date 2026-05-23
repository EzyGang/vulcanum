import type {
  ColumnInfo,
  ColumnsResponse,
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

export const listColumnsByKaneoId = async (kaneoProjectId: string): Promise<ColumnInfo[]> => {
  const response = await get<ColumnsResponse>('/projects/columns', {
    kaneo_project_id: kaneoProjectId
  });
  return response.columns;
};
