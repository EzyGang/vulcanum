use async_trait::async_trait;
use kaneo_cli::api::client::ApiClient;
use kaneo_cli::api::types::{BoardResponse, Column, Comment, Task};

use super::errors::{api_err, KaneoError};

const FETCH_TASKS_LIMIT: u32 = 200;

#[async_trait]
pub trait TaskFetcher: Send + Sync {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<Task>, KaneoError>;
}

#[derive(Clone)]
pub struct KaneoClient {
    pub instance: String,
    pub api_key: String,
}

#[async_trait]
impl TaskFetcher for KaneoClient {
    async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<Task>, KaneoError> {
        self.do_fetch_tasks_in_column(project_id, column_name).await
    }
}

impl KaneoClient {
    pub fn new(instance: String, api_key: String) -> Self {
        Self { instance, api_key }
    }

    fn build_client(&self) -> Result<ApiClient, KaneoError> {
        ApiClient::new(&self.instance, &self.api_key).map_err(api_err)
    }

    async fn do_fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_name: &str,
    ) -> Result<Vec<Task>, KaneoError> {
        let column_slug = slugify(column_name);
        let client = self.build_client()?;
        let path =
            format!("/task/tasks/{project_id}?limit={FETCH_TASKS_LIMIT}&status={column_slug}");

        let start = std::time::Instant::now();
        let result: Result<BoardResponse, KaneoError> = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);

        result.map(|board| filter_tasks_in_column(board, &column_slug))
    }

    /// Accepts both slugs and display names — normalizes to a slug internally.
    pub async fn update_task_status(
        &self,
        task_id: &str,
        new_status: &str,
    ) -> Result<(), KaneoError> {
        let new_status = slugify(new_status);
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct StatusBody {
            status: String,
        }

        let path = format!("/task/status/{task_id}");
        let start = std::time::Instant::now();
        let result = client
            .put(
                &path,
                &StatusBody {
                    status: new_status.to_owned(),
                },
            )
            .await
            .map(|_: Task| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }

    pub async fn add_comment(&self, task_id: &str, content: &str) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct CommentBody {
            content: String,
        }

        let path = format!("/comment/{task_id}");
        let start = std::time::Instant::now();
        let result = client
            .post(
                &path,
                &CommentBody {
                    content: content.to_owned(),
                },
            )
            .await
            .map(|_: Comment| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("POST", &path, duration_ms, &result);
        result
    }

    pub async fn fetch_columns(&self, project_id: &str) -> Result<Vec<Column>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/column/{project_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }
}

pub(crate) fn filter_tasks_in_column(board: BoardResponse, column_slug: &str) -> Vec<Task> {
    board
        .data
        .columns
        .into_iter()
        .find(|col| match col.status.as_deref() {
            Some(status) => status == column_slug,
            None => slugify(&col.name) == column_slug,
        })
        .map(|col| col.tasks)
        .unwrap_or_default()
}

pub fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_lowercase()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

pub(crate) fn log_kaneo_result<T>(
    method: &str,
    path: &str,
    duration_ms: i64,
    result: &Result<T, KaneoError>,
) {
    match result {
        Ok(_) => {
            tracing::info!(
                method = method,
                path = path,
                duration_ms = duration_ms,
                "Kaneo API call succeeded",
            );
        }
        Err(e) => {
            tracing::warn!(
                method = method,
                path = path,
                duration_ms = duration_ms,
                error = %e,
                "Kaneo API call failed",
            );
        }
    }
}
