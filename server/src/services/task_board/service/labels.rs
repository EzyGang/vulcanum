use super::*;

impl TaskBoardService {
    pub async fn add_task_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.add_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn remove_task_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        label_id: &str,
    ) -> Result<TaskLabelResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.remove_task_label(task_id, &label_id).await?;

        Ok(TaskLabelResponse {
            task_id: task_id.to_owned(),
            label_id,
        })
    }

    pub async fn delete_label(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        label_id: &str,
    ) -> Result<TaskLabelDeleteResponse, TaskBoardError> {
        let label_id = normalized_required(label_id, TaskBoardError::EmptyLabel)?;
        let provider = self.load_configured_provider(team_id, provider_id).await?;
        let client = IntegrationClient::from_provider(&provider);
        client.delete_label(&label_id).await?;

        Ok(TaskLabelDeleteResponse { label_id })
    }
}
