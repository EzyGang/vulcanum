use std::collections::HashSet;

use kaneo_cli::api::types::{Comment, CreateTaskBody};

use crate::services::providers::kaneo::client::types::{KaneoBoardResponse, KaneoTask};
use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

const FETCH_TASKS_LIMIT: u32 = 200;

impl KaneoClient {
    pub(crate) async fn fetch_board(
        &self,
        project_id: &str,
    ) -> Result<KaneoBoardResponse, KaneoError> {
        self.fetch_board_pages(project_id, None).await
    }

    pub(crate) async fn fetch_task(&self, task_id: &str) -> Result<KaneoTask, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/task/{task_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub(crate) async fn fetch_tasks_in_column(
        &self,
        project_id: &str,
        column_slug: &str,
    ) -> Result<(Vec<KaneoTask>, String), KaneoError> {
        let board = self
            .fetch_board_pages(project_id, Some(column_slug))
            .await?;
        let column_summaries = board
            .data
            .columns
            .iter()
            .map(|column| {
                format!(
                    "{}:{}:{}",
                    column.name,
                    column.status.as_deref().unwrap_or("<none>"),
                    column.tasks.len()
                )
            })
            .collect::<Vec<String>>();
        tracing::debug!(
            project_id = %project_id,
            requested_status = %column_slug,
            board_slug = %board.data.slug,
            columns = ?column_summaries,
            planned_tasks = board.data.planned_tasks.len(),
            archived_tasks = board.data.archived_tasks.len(),
            "received Kaneo task board",
        );

        let slug = board.data.slug.clone();
        let tasks = filter_tasks_in_column(board, column_slug);
        tracing::debug!(
            project_id = %project_id,
            requested_status = %column_slug,
            tasks_selected = tasks.len(),
            tasks = ?tasks
                .iter()
                .map(|task| format!("{}:{}:{}", task.id, task.status, task.title))
                .collect::<Vec<String>>(),
            "selected Kaneo tasks for Vulcanum polling",
        );
        Ok((tasks, slug))
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
    async fn fetch_board_pages(
        &self,
        project_id: &str,
        status: Option<&str>,
    ) -> Result<KaneoBoardResponse, KaneoError> {
        let client = self.build_client()?;
        let mut offset = 0;
        let mut merged: Option<KaneoBoardResponse> = None;
        let mut seen_task_ids = HashSet::new();

        loop {
            let path = match status {
                Some(status) => format!(
                    "/task/tasks/{project_id}?limit={FETCH_TASKS_LIMIT}&offset={offset}&status={status}"
                ),
                None => {
                    format!("/task/tasks/{project_id}?limit={FETCH_TASKS_LIMIT}&offset={offset}")
                }
            };

            let start = std::time::Instant::now();
            let result: Result<KaneoBoardResponse, KaneoError> =
                client.get(&path).await.map_err(api_err);
            let duration_ms = start.elapsed().as_millis() as i64;

            log_kaneo_result("GET", &path, duration_ms, &result);

            let mut page = result?;
            let page_task_count = board_task_count(&page);
            let new_task_count = match &mut merged {
                Some(merged) => merge_board_page(merged, &mut page, &mut seen_task_ids),
                None => {
                    collect_seen_task_ids(&page, &mut seen_task_ids);
                    merged = Some(page);
                    page_task_count
                }
            };

            if page_task_count < FETCH_TASKS_LIMIT as usize || new_task_count == 0 {
                break;
            }
            offset += FETCH_TASKS_LIMIT;
        }

        merged.ok_or_else(|| KaneoError::Api("empty paginated task response".to_owned()))
    }
}

fn board_task_count(board: &KaneoBoardResponse) -> usize {
    board
        .data
        .columns
        .iter()
        .map(|column| column.tasks.len())
        .sum::<usize>()
        + board.data.planned_tasks.len()
        + board.data.archived_tasks.len()
}

fn collect_seen_task_ids(board: &KaneoBoardResponse, seen_task_ids: &mut HashSet<String>) {
    for column in &board.data.columns {
        for task in &column.tasks {
            seen_task_ids.insert(task.id.clone());
        }
    }
    for task in board
        .data
        .planned_tasks
        .iter()
        .chain(&board.data.archived_tasks)
    {
        seen_task_ids.insert(task.id.clone());
    }
}

fn merge_board_page(
    merged: &mut KaneoBoardResponse,
    page: &mut KaneoBoardResponse,
    seen_task_ids: &mut HashSet<String>,
) -> usize {
    let mut new_task_count = 0;

    for page_column in &mut page.data.columns {
        let Some(merged_column) = merged
            .data
            .columns
            .iter_mut()
            .find(|column| column.id == page_column.id)
        else {
            new_task_count += push_new_tasks(
                &mut merged.data.planned_tasks,
                &mut page_column.tasks,
                seen_task_ids,
            );
            continue;
        };

        new_task_count += push_new_tasks(
            &mut merged_column.tasks,
            &mut page_column.tasks,
            seen_task_ids,
        );
    }

    new_task_count += push_new_tasks(
        &mut merged.data.planned_tasks,
        &mut page.data.planned_tasks,
        seen_task_ids,
    );
    new_task_count += push_new_tasks(
        &mut merged.data.archived_tasks,
        &mut page.data.archived_tasks,
        seen_task_ids,
    );

    new_task_count
}

fn push_new_tasks(
    target: &mut Vec<KaneoTask>,
    tasks: &mut Vec<KaneoTask>,
    seen_task_ids: &mut HashSet<String>,
) -> usize {
    let original_len = target.len();
    target.extend(
        tasks
            .drain(..)
            .filter(|task| seen_task_ids.insert(task.id.clone())),
    );
    target.len() - original_len
}

pub(crate) fn filter_tasks_in_column(
    board: KaneoBoardResponse,
    column_slug: &str,
) -> Vec<KaneoTask> {
    board
        .data
        .columns
        .into_iter()
        .flat_map(|column| column.tasks)
        .chain(board.data.planned_tasks)
        .chain(board.data.archived_tasks)
        .filter(|task| task.status == column_slug)
        .collect()
}
