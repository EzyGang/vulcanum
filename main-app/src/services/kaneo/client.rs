use kaneo_cli::api::client::ApiClient;
use kaneo_cli::api::types::{BoardResponse, Column, Comment, Task};

use super::errors::{api_err, KaneoError};

#[derive(Clone)]
pub struct KaneoClient {
    instance: String,
    api_key: String,
}

impl KaneoClient {
    pub fn new(instance: String, api_key: String) -> Self {
        Self { instance, api_key }
    }

    fn build_client(&self) -> Result<ApiClient, KaneoError> {
        ApiClient::new(&self.instance, &self.api_key).map_err(api_err)
    }

    pub async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_slug: &str,
    ) -> Result<Vec<Task>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/task/tasks/{project_id}?limit=50&status={column_slug}");
        let board: BoardResponse = client.get(&path).await.map_err(api_err)?;

        Ok(filter_tasks_in_column(board, column_slug))
    }

    pub async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct StatusBody {
            status: String,
        }

        client
            .put(
                &format!("/task/status/{task_id}"),
                &StatusBody {
                    status: new_status.to_owned(),
                },
            )
            .await
            .map(|_: Task| ())
            .map_err(api_err)
    }

    pub async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct CommentBody {
            content: String,
        }

        client
            .post(
                &format!("/comment/{task_id}"),
                &CommentBody {
                    content: content.to_owned(),
                },
            )
            .await
            .map(|_: Comment| ())
            .map_err(api_err)
    }

    pub async fn fetch_columns(&self, project_id: &str) -> Result<Vec<Column>, KaneoError> {
        let client = self.build_client()?;
        let columns: Vec<Column> = client
            .get(&format!("/column/{project_id}"))
            .await
            .map_err(api_err)?;

        Ok(columns)
    }
}

pub(crate) fn filter_tasks_in_column(board: BoardResponse, column_slug: &str) -> Vec<Task> {
    board
        .data
        .columns
        .into_iter()
        .find(|col| col.name.to_lowercase() == column_slug.to_lowercase())
        .map(|col| col.tasks)
        .unwrap_or_default()
}
