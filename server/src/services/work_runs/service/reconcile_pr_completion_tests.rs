use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::{
    CreateIntegrationLabelInput, CreateIntegrationTaskInput, IntegrationBoard, IntegrationColumn,
    IntegrationLabel, IntegrationProject, IntegrationTask, IntegrationType, IntegrationWorkspace,
    UpdateIntegrationLabelInput, UpdateIntegrationTaskInput,
};
use crate::services::providers::client::{IntegrationClient, IntegrationProviderClient};
use crate::test_helpers;

struct LabelFailingClient {
    current_status: String,
    status_updates: Arc<Mutex<Vec<(String, String)>>>,
}

#[async_trait]
impl IntegrationProviderClient for LabelFailingClient {
    fn provider_type(&self) -> IntegrationType {
        IntegrationType::Kaneo
    }

    async fn fetch_columns(
        &self,
        _project_id: &str,
    ) -> Result<Vec<IntegrationColumn>, IntegrationError> {
        Err(unexpected_call("fetch_columns"))
    }

    async fn fetch_board(&self, _project_id: &str) -> Result<IntegrationBoard, IntegrationError> {
        Err(unexpected_call("fetch_board"))
    }

    async fn fetch_task(&self, task_id: &str) -> Result<IntegrationTask, IntegrationError> {
        Ok(IntegrationTask {
            id: task_id.to_owned(),
            title: "Merged pull request".to_owned(),
            project_id: "project-1".to_owned(),
            description: None,
            status: self.current_status.clone(),
            priority: "medium".to_owned(),
            number: Some(1),
            project_slug: Some("project".to_owned()),
            assignee_name: None,
            created_at: "2026-07-25T00:00:00Z".to_owned(),
            updated_at: None,
            labels: Vec::new(),
        })
    }

    async fn fetch_tasks_in_column(
        &self,
        _project_id: &str,
        _column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        Err(unexpected_call("fetch_tasks_in_column"))
    }

    async fn create_task(
        &self,
        _input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        Err(unexpected_call("create_task"))
    }

    async fn update_task(
        &self,
        _input: UpdateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        Err(unexpected_call("update_task"))
    }

    async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError> {
        self.status_updates
            .lock()
            .await
            .push((task_id.to_owned(), new_status.to_owned()));
        Ok(())
    }

    async fn add_comment(&self, _task_id: &str, _content: &str) -> Result<(), IntegrationError> {
        Err(unexpected_call("add_comment"))
    }

    async fn update_task_description(
        &self,
        _task_id: &str,
        _description: &str,
    ) -> Result<(), IntegrationError> {
        Err(unexpected_call("update_task_description"))
    }

    async fn lookup_project(
        &self,
        _project_id: &str,
    ) -> Result<IntegrationProject, IntegrationError> {
        Err(unexpected_call("lookup_project"))
    }

    async fn fetch_workspaces(&self) -> Result<Vec<IntegrationWorkspace>, IntegrationError> {
        Err(unexpected_call("fetch_workspaces"))
    }

    async fn fetch_projects(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<IntegrationProject>, IntegrationError> {
        Err(unexpected_call("fetch_projects"))
    }

    async fn fetch_labels(
        &self,
        _workspace_id: &str,
    ) -> Result<Vec<IntegrationLabel>, IntegrationError> {
        Err(IntegrationError::Other("label API unavailable".to_owned()))
    }

    async fn create_label(
        &self,
        _input: CreateIntegrationLabelInput,
    ) -> Result<IntegrationLabel, IntegrationError> {
        Err(unexpected_call("create_label"))
    }

    async fn update_label(
        &self,
        _input: UpdateIntegrationLabelInput,
    ) -> Result<IntegrationLabel, IntegrationError> {
        Err(unexpected_call("update_label"))
    }

    async fn delete_label(&self, _label_id: &str) -> Result<(), IntegrationError> {
        Err(unexpected_call("delete_label"))
    }

    async fn add_task_label(
        &self,
        _task_id: &str,
        _label_id: &str,
    ) -> Result<(), IntegrationError> {
        Err(unexpected_call("add_task_label"))
    }

    async fn remove_task_label(
        &self,
        _task_id: &str,
        _label_id: &str,
    ) -> Result<(), IntegrationError> {
        Err(unexpected_call("remove_task_label"))
    }
}

#[sqlx::test]
async fn merged_pr_moves_task_when_lifecycle_label_sync_fails(pool: sqlx::PgPool) {
    let project_config_id =
        test_helpers::project_configs::insert_project_config(&pool, "project-1").await;
    let state = test_helpers::state::build_state(pool).await;
    let config = state
        .project_configs
        .find_by_id(project_config_id)
        .await
        .expect("load project config");
    let status_updates = Arc::new(Mutex::new(Vec::new()));
    let client = IntegrationClient::new(LabelFailingClient {
        current_status: config.review_column.clone(),
        status_updates: status_updates.clone(),
    });

    let moved = state
        .jobs
        .sync_task_to_done(&config, "task-1", &client)
        .await
        .expect("label failure must not block status transition");

    assert!(moved);
    assert_eq!(
        status_updates.lock().await.as_slice(),
        &[("task-1".to_owned(), "done".to_owned())]
    );
}

fn unexpected_call(operation: &str) -> IntegrationError {
    IntegrationError::Other(format!("unexpected provider operation: {operation}"))
}
