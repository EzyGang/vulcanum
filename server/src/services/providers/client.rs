use std::sync::Arc;

use async_trait::async_trait;

use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::{
    CreateIntegrationTaskInput, IntegrationBoard, IntegrationColumn, IntegrationProject,
    IntegrationTask, IntegrationType, IntegrationWorkspace,
};
use crate::services::providers::kaneo::client::KaneoClient;

#[derive(Clone)]
pub struct IntegrationClient {
    provider_type: IntegrationType,
    inner: Arc<dyn IntegrationProviderClient>,
}

#[async_trait]
pub trait IntegrationProviderClient: Send + Sync {
    fn provider_type(&self) -> IntegrationType;

    async fn fetch_columns(
        &self,
        project_id: &str,
    ) -> Result<Vec<IntegrationColumn>, IntegrationError>;

    async fn fetch_board(&self, project_id: &str) -> Result<IntegrationBoard, IntegrationError>;

    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError>;

    async fn create_task(
        &self,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError>;

    async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError>;

    async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), IntegrationError>;

    async fn update_task_description(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), IntegrationError>;

    async fn lookup_project(
        &self,
        project_id: &str,
    ) -> Result<IntegrationProject, IntegrationError>;

    async fn fetch_workspaces(&self) -> Result<Vec<IntegrationWorkspace>, IntegrationError>;

    async fn fetch_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<IntegrationProject>, IntegrationError>;
}

impl IntegrationClient {
    #[must_use]
    pub fn from_provider(provider: &IntegrationProvider) -> Self {
        match provider.provider_type {
            IntegrationType::Kaneo => {
                Self::new_kaneo(provider.instance_url.clone(), provider.api_key.clone())
            }
        }
    }

    #[must_use]
    fn new_kaneo(instance: String, api_key: String) -> Self {
        Self::new(KaneoClient::new(instance, api_key))
    }

    #[must_use]
    pub fn new<T>(client: T) -> Self
    where
        T: IntegrationProviderClient + 'static,
    {
        let provider_type = client.provider_type();

        Self {
            provider_type,
            inner: Arc::new(client),
        }
    }

    #[must_use]
    pub fn provider_type(&self) -> IntegrationType {
        self.provider_type
    }

    pub async fn fetch_columns(
        &self,
        project_id: &str,
    ) -> Result<Vec<IntegrationColumn>, IntegrationError> {
        self.inner.fetch_columns(project_id).await
    }

    pub async fn fetch_board(
        &self,
        project_id: &str,
    ) -> Result<IntegrationBoard, IntegrationError> {
        self.inner.fetch_board(project_id).await
    }

    pub async fn create_task(
        &self,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        self.inner.create_task(input).await
    }

    pub async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError> {
        self.inner.update_task_status(task_id, new_status).await
    }

    pub async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), IntegrationError> {
        self.inner.add_comment(task_id, content).await
    }

    pub async fn update_task_description(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), IntegrationError> {
        self.inner
            .update_task_description(task_id, description)
            .await
    }

    pub async fn lookup_project(
        &self,
        project_id: &str,
    ) -> Result<IntegrationProject, IntegrationError> {
        self.inner.lookup_project(project_id).await
    }

    pub async fn fetch_workspaces(&self) -> Result<Vec<IntegrationWorkspace>, IntegrationError> {
        self.inner.fetch_workspaces().await
    }

    pub async fn fetch_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<IntegrationProject>, IntegrationError> {
        self.inner.fetch_projects(workspace_id).await
    }
}

#[async_trait]
impl TaskFetcher for IntegrationClient {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        self.inner
            .fetch_tasks_in_column(project_id, column_name)
            .await
    }
}

#[async_trait]
pub trait TaskFetcher: Send + Sync {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError>;
}
