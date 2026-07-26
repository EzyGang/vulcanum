use super::{
    collect_board_task_refs, IntegrationBoard, IntegrationClient, IntegrationProvider,
    IntegrationTask, ProjectConfig, ProjectConfigsError, TaskBoardError, TaskBoardService,
    TaskBoardTaskAugmentation, Uuid,
};

impl TaskBoardService {
    pub(super) async fn task_augmentations(
        &self,
        team_id: Uuid,
        project_config_id: Uuid,
        board: &IntegrationBoard,
    ) -> Result<Vec<TaskBoardTaskAugmentation>, TaskBoardError> {
        let task_refs = collect_board_task_refs(board);
        if task_refs.is_empty() {
            return Ok(Vec::new());
        }

        self.task_augmentations_repo
            .list_for_task_refs(&self.db, team_id, project_config_id, &task_refs)
            .await
            .map_err(TaskBoardError::from)
    }

    pub(super) async fn load_project_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<(ProjectConfig, IntegrationProvider), TaskBoardError> {
        let project_config = self
            .project_configs_repo
            .find_by_provider_project(&self.db, team_id, provider_id, external_project_id)
            .await?
            .ok_or(ProjectConfigsError::NotFound)?;
        let provider = self.load_provider(team_id, provider_id).await?;

        Ok((project_config, provider))
    }

    pub(super) async fn load_task_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
    ) -> Result<(IntegrationClient, IntegrationTask), TaskBoardError> {
        let provider = self.load_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        let task = client.fetch_task(task_id).await?;
        self.project_configs_repo
            .find_by_provider_project(&self.db, team_id, provider_id, &task.project_id)
            .await?
            .ok_or(ProjectConfigsError::NotFound)?;

        Ok((client, task))
    }

    pub(super) async fn load_configured_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
    ) -> Result<IntegrationProvider, TaskBoardError> {
        let provider = self.load_provider(team_id, provider_id).await?;
        let configs = self
            .project_configs_repo
            .list_all(&self.db, team_id)
            .await?;
        if !configs
            .iter()
            .any(|config| config.provider_id == Some(provider_id))
        {
            return Err(ProjectConfigsError::NotFound.into());
        }

        Ok(provider)
    }

    async fn load_provider(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
    ) -> Result<IntegrationProvider, TaskBoardError> {
        self.providers_repo
            .find_by_id(&self.db, provider_id, team_id)
            .await
            .map_err(TaskBoardError::from)
    }
}
