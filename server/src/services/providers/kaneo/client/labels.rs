use kaneo_cli::api::types::{CreateLabelBody, Label};

use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

#[derive(serde::Serialize)]
struct TaskLabelBody {
    #[serde(rename = "taskId")]
    task_id: String,
}
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

    pub async fn delete_label(&self, label_id: &str) -> Result<(), KaneoError> {
        let path = format!("/label/{label_id}");

        let start = std::time::Instant::now();
        let result = self.delete_resource(&path).await;
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("DELETE", &path, duration_ms, &result);
        result
    }

    pub async fn add_task_label(&self, task_id: &str, label_id: &str) -> Result<(), KaneoError> {
        let client = self.build_client()?;

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
    async fn delete_resource(&self, path: &str) -> Result<(), KaneoError> {
        let url = format!(
            "https://{}/api{}",
            self.instance.trim_end_matches('/'),
            path,
        );
        let response = reqwest::Client::new()
            .delete(url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(api_err)?;
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable body>".to_owned());
        let message = provider_error_message(&body);

        Err(KaneoError::Api(format!("{status}: {message}")))
    }
}

fn provider_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| body.to_owned())
}
