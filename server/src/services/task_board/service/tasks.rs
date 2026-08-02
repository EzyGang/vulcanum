use super::{
    default_task_status, normalized_required, CreateIntegrationTaskInput, CreateTaskRequest,
    CreateTaskResponse, IntegrationClient, MoveTaskResponse, TaskBoardError, TaskBoardService,
    UpdateIntegrationTaskInput, UpdateTaskRequest, UpdateTaskResponse, Uuid, DEFAULT_PRIORITY,
};

impl TaskBoardService {
    pub async fn create_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        request: CreateTaskRequest,
    ) -> Result<CreateTaskResponse, TaskBoardError> {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let (_, provider) = self
            .load_project_provider(team_id, provider_id, external_project_id)
            .await?;
        let client = IntegrationClient::from_provider(&provider);
        let status = match request.status.as_deref().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_owned(),
            _ => default_task_status(&client, external_project_id).await?,
        };
        let priority = request
            .priority
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_PRIORITY)
            .to_owned();

        let task = client
            .create_task(CreateIntegrationTaskInput {
                project_id: external_project_id.to_owned(),
                title,
                body: request.body,
                status,
                priority,
            })
            .await?;

        Ok(CreateTaskResponse { task })
    }

    pub async fn update_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        request: UpdateTaskRequest,
    ) -> Result<UpdateTaskResponse, TaskBoardError> {
        let title = normalized_required(&request.title, TaskBoardError::EmptyTitle)?;
        let (client, task) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;
        let task = client
            .update_task(task_update_input(task, title, request.body))
            .await
            .map_err(TaskBoardError::TaskUpdate)?;

        Ok(UpdateTaskResponse { task })
    }

    pub async fn move_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        status: &str,
    ) -> Result<MoveTaskResponse, TaskBoardError> {
        let next_status = normalized_required(status, TaskBoardError::EmptyStatus)?;
        let (client, _) = self
            .load_task_provider(team_id, provider_id, task_id)
            .await?;

        client.update_task_status(task_id, &next_status).await?;

        Ok(MoveTaskResponse {
            task_id: task_id.to_owned(),
            status: next_status,
        })
    }
}

pub(crate) fn task_update_input(
    current: crate::models::providers::model::IntegrationTask,
    title: String,
    body: String,
) -> UpdateIntegrationTaskInput {
    UpdateIntegrationTaskInput {
        task_id: current.id,
        title,
        body,
        status: current.status,
        priority: current.priority,
        project_id: current.project_id,
        position: current.position.unwrap_or(0.0),
        due_date: current.due_date.unwrap_or_default(),
        start_date: current.start_date.unwrap_or_default(),
        user_id: current.assignee_id.unwrap_or_default(),
    }
}
