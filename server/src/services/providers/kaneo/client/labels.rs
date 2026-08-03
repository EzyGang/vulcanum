use kaneo_cli::api::types::{CreateLabelBody, Label};

use crate::services::providers::kaneo::client::types::KaneoTask;
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
    pub(super) async fn fetch_label(&self, label_id: &str) -> Result<Label, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/label/{label_id}");

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
        task_id: Option<&str>,
    ) -> Result<Label, KaneoError> {
        let client = self.build_client()?;
        let path = "/label";
        let body = create_label_body(workspace_id, name, color, task_id);

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
        let client = self.build_client()?;
        let path = format!("/label/{label_id}");

        let start = std::time::Instant::now();
        let result = client
            .delete(&path)
            .await
            .map(|_: serde_json::Value| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("DELETE", &path, duration_ms, &result);
        result
    }

    pub async fn add_task_label(&self, task_id: &str, label_id: &str) -> Result<(), KaneoError> {
        let label = self.fetch_label(label_id).await?;
        if label.task_id.as_deref() == Some(task_id) {
            return Ok(());
        }

        let task = self.fetch_task(task_id).await?;
        if !task_label_ids_by_name(&task, &label.name).is_empty() {
            return Ok(());
        }

        let workspace_id = label
            .workspace_id
            .as_deref()
            .ok_or_else(|| KaneoError::Api(format!("label {label_id} has no workspace")))?;
        self.create_label(workspace_id, &label.name, &label.color, Some(task_id))
            .await
            .map(|_| ())
    }

    pub async fn remove_task_label(&self, task_id: &str, label_id: &str) -> Result<(), KaneoError> {
        let label = self.fetch_label(label_id).await?;
        if label.task_id.as_deref() == Some(task_id) {
            return self.delete_label(label_id).await;
        }

        let task = self.fetch_task(task_id).await?;
        for task_label_id in task_label_ids_by_name(&task, &label.name) {
            self.delete_label(&task_label_id).await?;
        }

        Ok(())
    }
}

pub(super) fn create_label_body(
    workspace_id: &str,
    name: &str,
    color: &str,
    task_id: Option<&str>,
) -> CreateLabelBody {
    CreateLabelBody {
        name: name.to_owned(),
        color: color.to_owned(),
        workspace_id: workspace_id.to_owned(),
        task_id: task_id.map(str::to_owned),
    }
}

pub(super) fn task_label_ids_by_name(task: &KaneoTask, label_name: &str) -> Vec<String> {
    task.labels
        .iter()
        .filter(|label| label.name == label_name)
        .map(|label| label.id.clone())
        .collect()
}
