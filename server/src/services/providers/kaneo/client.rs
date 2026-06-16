use kaneo_cli::api::client::ApiClient;
use kaneo_cli::api::types::{BoardResponse, Column, Comment, Project, Task};

use super::errors::{api_err, KaneoError};

const FETCH_TASKS_LIMIT: u32 = 200;

#[derive(Clone)]
pub struct KaneoClient {
    pub instance: String,
    pub api_key: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaneoWorkspace {
    pub id: String,
    pub name: String,
    pub slug: String,
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
    ) -> Result<(Vec<Task>, String), KaneoError> {
        let client = self.build_client()?;
        let path =
            format!("/task/tasks/{project_id}?limit={FETCH_TASKS_LIMIT}&status={column_slug}");

        let start = std::time::Instant::now();
        let result: Result<BoardResponse, KaneoError> = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);

        result.map(|board| {
            let slug = board.data.slug.clone();
            let tasks = filter_tasks_in_column(board, column_slug);
            (tasks, slug)
        })
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

    pub async fn update_task_description(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct DescriptionBody {
            description: String,
        }

        let path = format!("/task/{task_id}");
        let start = std::time::Instant::now();
        let result = client
            .put(
                &path,
                &DescriptionBody {
                    description: description.to_owned(),
                },
            )
            .await
            .map(|_: Task| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
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

    pub async fn lookup_project(&self, project_id: &str) -> Result<Project, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/project/{project_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub async fn fetch_workspaces(&self) -> Result<Vec<KaneoWorkspace>, KaneoError> {
        let client = self.build_client()?;
        let path = "/auth/organization/list";

        let start = std::time::Instant::now();
        let result = client.get(path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", path, duration_ms, &result);
        result
    }

    pub async fn fetch_projects(&self, workspace_id: &str) -> Result<Vec<Project>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/project?workspaceId={workspace_id}");

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
        .find(|col| col.status.as_deref() == Some(column_slug))
        .map(|col| col.tasks)
        .unwrap_or_default()
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
