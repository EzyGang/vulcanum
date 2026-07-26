use super::*;

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
