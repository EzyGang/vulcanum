use kaneo_cli::api::types::{CreateLabelBody, Label};

use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

impl KaneoClient {
    pub async fn fetch_labels(&self, workspace_id: &str) -> Result<Vec<Label>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/label/workspace/{workspace_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub async fn fetch_task_labels(&self, task_id: &str) -> Result<Vec<Label>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/label/task/{task_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub async fn create_label(
        &self,
        workspace_id: &str,
        name: &str,
        color: &str,
    ) -> Result<Label, KaneoError> {
        let client = self.build_client()?;
        let path = "/label";
        let body = CreateLabelBody {
            name: name.to_owned(),
            color: color.to_owned(),
            workspace_id: workspace_id.to_owned(),
            task_id: None,
        };

        let start = std::time::Instant::now();
        let result = client.post(path, &body).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("POST", path, duration_ms, &result);
        result
    }

    pub async fn update_label(
        &self,
        label_id: &str,
        name: Option<&str>,
        color: Option<&str>,
    ) -> Result<Label, KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct LabelBody {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            color: Option<String>,
        }

        let path = format!("/label/{label_id}");
        let body = LabelBody {
            name: name.map(str::to_owned),
            color: color.map(str::to_owned),
        };
        let start = std::time::Instant::now();
        let result = client.put(&path, &body).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }

    pub async fn delete_label(&self, label_id: &str) -> Result<Label, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/label/{label_id}");

        let start = std::time::Instant::now();
        let result = client.delete(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("DELETE", &path, duration_ms, &result);
        result
    }

    pub async fn add_task_label(&self, task_id: &str, label_id: &str) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct TaskLabelBody {
            #[serde(rename = "taskId")]
            task_id: String,
        }

        let path = format!("/label/{label_id}/task");
        let start = std::time::Instant::now();
        let result = client
            .put(
                &path,
                &TaskLabelBody {
                    task_id: task_id.to_owned(),
                },
            )
            .await
            .map(|_: serde_json::Value| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }

    pub async fn remove_task_label(&self, task_id: &str, label_id: &str) -> Result<(), KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct TaskLabelBody {
            #[serde(rename = "taskId")]
            task_id: String,
        }

        let path = format!("/label/{label_id}/task");
        let start = std::time::Instant::now();
        let result = client
            .delete_json(
                &path,
                &TaskLabelBody {
                    task_id: task_id.to_owned(),
                },
            )
            .await
            .map(|_: serde_json::Value| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("DELETE", &path, duration_ms, &result);
        result
    }
}
