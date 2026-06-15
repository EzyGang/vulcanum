use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{
    ColumnInfo, LookupProjectResult, ProjectInfo, WorkspaceInfo,
};
use crate::services::project_configs::service::ProjectConfigsService;

impl ProjectConfigsService {
    pub async fn lookup_project(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
        external_project_id: &str,
    ) -> Result<LookupProjectResult, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;

        let project = client
            .lookup_project(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        let columns = client
            .fetch_columns(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(LookupProjectResult {
            id: project.id,
            name: project.name,
            slug: project.slug,
            columns: columns.iter().map(ColumnInfo::from).collect(),
        })
    }

    pub async fn fetch_workspaces(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
    ) -> Result<Vec<WorkspaceInfo>, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;
        let workspaces = client
            .fetch_workspaces()
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(workspaces.into_iter().map(WorkspaceInfo::from).collect())
    }

    pub async fn fetch_projects(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
        workspace_id: &str,
    ) -> Result<Vec<ProjectInfo>, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;
        let projects = client
            .fetch_projects(workspace_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(projects.into_iter().map(ProjectInfo::from).collect())
    }
}
