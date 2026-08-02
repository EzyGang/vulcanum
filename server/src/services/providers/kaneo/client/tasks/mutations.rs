use super::{
    api_err, log_kaneo_result, Comment, CreateTaskBody, KaneoClient, KaneoError, KaneoTask,
};

use crate::models::providers::model::UpdateIntegrationTaskInput;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateTaskBody {
    title: String,
    description: String,
    priority: String,
    status: String,
    project_id: String,
    position: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

#[must_use]
pub(crate) fn update_task_request(input: &UpdateIntegrationTaskInput) -> (String, UpdateTaskBody) {
    (
        format!("/task/{}", input.task_id),
        UpdateTaskBody {
            title: input.title.clone(),
            description: input.body.clone(),
            priority: input.priority.clone(),
            status: input.status.clone(),
            project_id: input.project_id.clone(),
            position: input.position,
            due_date: input.due_date.clone(),
            start_date: input.start_date.clone(),
            user_id: input.user_id.clone(),
        },
    )
}

impl KaneoClient {
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
            .map(|_: KaneoTask| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }

    pub(crate) async fn create_task(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
        status: &str,
        priority: &str,
    ) -> Result<KaneoTask, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/task/{project_id}");
        let body = CreateTaskBody {
            title: title.to_owned(),
            description: description.to_owned(),
            priority: priority.to_owned(),
            status: status.to_owned(),
            due_date: None,
            start_date: None,
            user_id: None,
        };

        let start = std::time::Instant::now();
        let result = client.post(&path, &body).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("POST", &path, duration_ms, &result);
        result
    }

    pub(crate) async fn update_task(
        &self,
        input: &UpdateIntegrationTaskInput,
    ) -> Result<KaneoTask, KaneoError> {
        let client = self.build_client()?;
        let (path, body) = update_task_request(input);
        let start = std::time::Instant::now();
        let result = client.put(&path, &body).await.map_err(api_err);
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

        let path = Self::task_description_path(task_id);
        let start = std::time::Instant::now();
        let result = client
            .put(
                &path,
                &DescriptionBody {
                    description: description.to_owned(),
                },
            )
            .await
            .map(|_: KaneoTask| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }

    #[must_use]
    pub(crate) fn task_description_path(task_id: &str) -> String {
        format!("/task/description/{task_id}")
    }
}
