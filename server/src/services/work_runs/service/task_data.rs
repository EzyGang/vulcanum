use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;

#[derive(Debug, Clone, Default)]
pub(crate) struct TaskRenderData {
    pub title: String,
    pub body: String,
}

impl From<IntegrationTask> for TaskRenderData {
    fn from(task: IntegrationTask) -> Self {
        Self {
            title: task.title,
            body: task.description.unwrap_or_default(),
        }
    }
}

impl WorkRunsService {
    pub(crate) async fn fetch_task_render_data(
        &self,
        config: &ProjectConfig,
        task_ref: &str,
    ) -> Result<TaskRenderData, WorkRunsError> {
        let Some(provider_id) = config.provider_id else {
            return Ok(TaskRenderData::default());
        };

        let provider = self
            .providers_repo
            .find_by_id(&self.db, provider_id, config.team_id)
            .await?;
        let task = IntegrationClient::from_provider(&provider)
            .fetch_task(&config.external_project_id, task_ref)
            .await
            .map_err(|error| {
                IntegrationError::Other(format!(
                    "failed to fetch task {task_ref} from provider {provider_id}: {error}"
                ))
            })?;

        Ok(task.into())
    }
}
