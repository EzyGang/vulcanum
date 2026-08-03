use super::{
    project_config_to_provider_project, IntegrationClient, TaskBoardError, TaskBoardResponse,
    TaskBoardService, TaskProviderProject, Uuid,
};

impl TaskBoardService {
    pub async fn list_projects(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<TaskProviderProject>, TaskBoardError> {
        let configs = self
            .project_configs_repo
            .list_all(&self.db, team_id)
            .await?;
        Ok(configs
            .into_iter()
            .filter_map(project_config_to_provider_project)
            .collect())
    }

    pub async fn get_board(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<TaskBoardResponse, TaskBoardError> {
        let (project_config, provider) = self
            .load_project_provider(team_id, provider_id, external_project_id)
            .await?;
        let client = IntegrationClient::from_provider(&provider);
        let mut board = client.fetch_board(external_project_id).await?;
        let project = client.lookup_project(external_project_id).await?;
        if let Some(workspace_id) = project.workspace_id.as_deref() {
            board.labels = client
                .fetch_labels(workspace_id)
                .await?
                .into_iter()
                .filter(|label| label.task_id.is_none())
                .collect();
        }
        let task_augmentations = self
            .task_augmentations(team_id, project_config.id, &board)
            .await?;
        let project_usage = self
            .project_usage_repo
            .summary(&self.db, project_config.id)
            .await?;

        Ok(TaskBoardResponse {
            provider_id: provider.id,
            provider_type: provider.provider_type,
            board,
            project_usage,
            task_augmentations,
        })
    }
}
