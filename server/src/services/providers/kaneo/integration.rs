use async_trait::async_trait;

use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::{
    CreateIntegrationLabelInput, CreateIntegrationTaskInput, IntegrationBoard, IntegrationColumn,
    IntegrationLabel, IntegrationProject, IntegrationTask, IntegrationType, IntegrationWorkspace,
    UpdateIntegrationLabelInput, UpdateIntegrationTaskInput,
};
use crate::services::providers::client::IntegrationProviderClient;
use crate::services::providers::kaneo::client::KaneoClient;
use crate::services::providers::kaneo::mapping::{
    kaneo_board_to_integration, kaneo_column_to_integration, kaneo_label_to_integration,
    kaneo_task_to_integration,
};

#[async_trait]
impl IntegrationProviderClient for KaneoClient {
    fn provider_type(&self) -> IntegrationType {
        IntegrationType::Kaneo
    }

    async fn fetch_columns(
        &self,
        project_id: &str,
    ) -> Result<Vec<IntegrationColumn>, IntegrationError> {
        let columns = KaneoClient::fetch_columns(self, project_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(columns.iter().map(kaneo_column_to_integration).collect())
    }

    async fn fetch_board(&self, project_id: &str) -> Result<IntegrationBoard, IntegrationError> {
        let board = KaneoClient::fetch_board(self, project_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(kaneo_board_to_integration(board))
    }

    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        let (tasks, project_slug) =
            KaneoClient::fetch_tasks_in_column(self, project_id, column_name)
                .await
                .map_err(IntegrationError::from)?;

        Ok(tasks
            .iter()
            .map(|task| kaneo_task_to_integration(task, Some(project_slug.as_str())))
            .collect())
    }

    async fn fetch_task(
        &self,
        project_id: &str,
        task_id: &str,
    ) -> Result<IntegrationTask, IntegrationError> {
        let (task, project_slug) = KaneoClient::fetch_task(self, project_id, task_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(kaneo_task_to_integration(
            &task,
            Some(project_slug.as_str()),
        ))
    }

    async fn create_task(
        &self,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        let task = KaneoClient::create_task(
            self,
            &input.project_id,
            &input.title,
            &input.body,
            &input.status,
            &input.priority,
        )
        .await
        .map_err(IntegrationError::from)?;

        Ok(kaneo_task_to_integration(&task, None))
    }

    async fn update_task(
        &self,
        input: UpdateIntegrationTaskInput,
    ) -> Result<IntegrationTask, IntegrationError> {
        let task = KaneoClient::update_task(self, &input.task_id, &input.title, &input.body)
            .await
            .map_err(IntegrationError::from)?;

        Ok(kaneo_task_to_integration(&task, None))
    }

    async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), IntegrationError> {
        KaneoClient::update_task_status(self, task_id, new_status)
            .await
            .map_err(IntegrationError::from)
    }

    async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), IntegrationError> {
        KaneoClient::add_comment(self, task_id, content)
            .await
            .map_err(IntegrationError::from)
    }

    async fn update_task_description(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), IntegrationError> {
        KaneoClient::update_task_description(self, task_id, description)
            .await
            .map_err(IntegrationError::from)
    }

    async fn lookup_project(
        &self,
        project_id: &str,
    ) -> Result<IntegrationProject, IntegrationError> {
        let project = KaneoClient::lookup_project(self, project_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(IntegrationProject {
            id: project.id,
            name: project.name,
            slug: project.slug,
            workspace_id: Some(project.workspace_id),
        })
    }

    async fn fetch_workspaces(&self) -> Result<Vec<IntegrationWorkspace>, IntegrationError> {
        let workspaces = KaneoClient::fetch_workspaces(self)
            .await
            .map_err(IntegrationError::from)?;

        Ok(workspaces
            .into_iter()
            .map(|workspace| IntegrationWorkspace {
                id: workspace.id,
                name: workspace.name,
            })
            .collect())
    }

    async fn fetch_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<IntegrationProject>, IntegrationError> {
        let projects = KaneoClient::fetch_projects(self, workspace_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(projects
            .into_iter()
            .map(|project| IntegrationProject {
                id: project.id,
                name: project.name,
                slug: project.slug,
                workspace_id: Some(project.workspace_id),
            })
            .collect())
    }

    async fn fetch_labels(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<IntegrationLabel>, IntegrationError> {
        let labels = KaneoClient::fetch_labels(self, workspace_id)
            .await
            .map_err(IntegrationError::from)?;

        Ok(labels.iter().map(kaneo_label_to_integration).collect())
    }

    async fn create_label(
        &self,
        input: CreateIntegrationLabelInput,
    ) -> Result<IntegrationLabel, IntegrationError> {
        let label = KaneoClient::create_label(self, &input.workspace_id, &input.name, &input.color)
            .await
            .map_err(IntegrationError::from)?;

        Ok(kaneo_label_to_integration(&label))
    }

    async fn update_label(
        &self,
        input: UpdateIntegrationLabelInput,
    ) -> Result<IntegrationLabel, IntegrationError> {
        let label = KaneoClient::update_label(
            self,
            &input.label_id,
            input.name.as_deref(),
            input.color.as_deref(),
        )
        .await
        .map_err(IntegrationError::from)?;

        Ok(kaneo_label_to_integration(&label))
    }

    async fn delete_label(&self, label_id: &str) -> Result<(), IntegrationError> {
        KaneoClient::delete_label(self, label_id)
            .await
            .map_err(IntegrationError::from)
    }

    async fn add_task_label(&self, task_id: &str, label_id: &str) -> Result<(), IntegrationError> {
        KaneoClient::add_task_label(self, task_id, label_id)
            .await
            .map_err(IntegrationError::from)
    }

    async fn remove_task_label(
        &self,
        task_id: &str,
        label_id: &str,
    ) -> Result<(), IntegrationError> {
        KaneoClient::remove_task_label(self, task_id, label_id)
            .await
            .map_err(IntegrationError::from)
    }
}
