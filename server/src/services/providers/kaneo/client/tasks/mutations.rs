use super::{
    api_err, log_kaneo_result, Comment, CreateTaskBody, KaneoClient, KaneoError, KaneoTask,
};

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
        task_id: &str,
        title: &str,
        description: &str,
    ) -> Result<KaneoTask, KaneoError> {
        let client = self.build_client()?;

        #[derive(serde::Serialize)]
        struct TaskBody {
            title: String,
            description: String,
        }

        let path = format!("/task/{task_id}");
        let body = TaskBody {
            title: title.to_owned(),
            description: description.to_owned(),
        };
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
            .map(|_: KaneoTask| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("PUT", &path, duration_ms, &result);
        result
    }
}
