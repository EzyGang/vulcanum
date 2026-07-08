use std::sync::Arc;

use async_trait::async_trait;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::providers::client::TaskFetcher;
use crate::services::work_runs::service::spawn_review::{upsert_pr_block, ReviewSpawnOutcome};
use crate::test_helpers;

#[test]
fn upsert_pr_block_appends_block() {
    let body = "Existing task body";
    let result = upsert_pr_block(
        body,
        &["https://github.com/acme/widgets/pull/42".to_owned()],
    );

    assert!(result.starts_with(body));
    assert!(result.contains("<!-- vulcanum:prs:start -->"));
    assert!(result.contains("- https://github.com/acme/widgets/pull/42"));
    assert!(result.contains("<!-- vulcanum:prs:end -->"));
}

#[test]
fn upsert_pr_block_replaces_existing_block() {
    let body = "Before\n<!-- vulcanum:prs:start -->\nold\n<!-- vulcanum:prs:end -->\nAfter";
    let result = upsert_pr_block(
        body,
        &["https://github.com/acme/widgets/pull/43".to_owned()],
    );

    assert!(result.contains("Before"));
    assert!(result.contains("After"));
    assert!(!result.contains("old"));
    assert!(result.contains("- https://github.com/acme/widgets/pull/43"));
}

struct StaticTaskFetcher {
    task: IntegrationTask,
}

#[async_trait]
impl TaskFetcher for StaticTaskFetcher {
    async fn fetch_tasks_in_column(
        &self,
        _project_id: &str,
        _column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        Ok(vec![self.task.clone()])
    }

    async fn fetch_task(&self, _task_id: &str) -> Result<IntegrationTask, IntegrationError> {
        Ok(self.task.clone())
    }

    async fn update_task_status(
        &self,
        _task_id: &str,
        _new_status: &str,
    ) -> Result<(), IntegrationError> {
        Ok(())
    }
}

fn static_task(title: &str) -> Arc<StaticTaskFetcher> {
    Arc::new(StaticTaskFetcher {
        task: IntegrationTask {
            id: "task-review-metadata".to_owned(),
            title: title.to_owned(),
            project_id: "kaneo-review-metadata".to_owned(),
            description: Some(
                "Implementation finished in https://github.com/acme/app/pull/17".to_owned(),
            ),
            status: "in-progress".to_owned(),
            priority: "medium".to_owned(),
            number: Some(1),
            project_slug: Some("WRONG".to_owned()),
            assignee_name: None,
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: None,
            labels: Vec::new(),
        },
    })
}

#[sqlx::test]
async fn spawn_review_inherits_parent_task_metadata(pool: sqlx::PgPool) {
    let svc = test_helpers::build_state(pool.clone())
        .await
        .jobs
        .with_task_fetcher(static_task(
            "Fetched title must not replace parent metadata",
        ));
    let project_config_id =
        test_helpers::insert_project_config(&pool, "kaneo-review-metadata").await;
    sqlx::query!(
        "UPDATE project_configs SET review_enabled = true WHERE id = $1",
        project_config_id,
    )
    .execute(&pool)
    .await
    .expect("Should enable review spawning");
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, $2, $3, $4)",
        project_config_id,
        "acme/app",
        "https://github.com/acme/app",
        0_i32,
    )
    .execute(&pool)
    .await
    .expect("Should insert project repo");
    let run = WorkRunsRepository::new()
        .insert_work_run(
            &pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-review-metadata".to_owned(),
                task_title: Some("Fix inherited review metadata".to_owned()),
                task_slug: Some("VLC-409".to_owned()),
                project_config_id,
                repo_full_names: vec!["acme/app".to_owned()],
                status: WorkRunStatus::Completed,
                work_type: WorkRunType::Implementation,
                parent_work_run_id: None,
                review_target_pr_url: None,
                review_target_repo_full_name: None,
            },
        )
        .await
        .expect("Should insert parent implementation run");
    let pr_url = "https://github.com/acme/app/pull/17".to_owned();

    let outcome = svc
        .attach_prs_and_spawn_reviews(&run, std::slice::from_ref(&pr_url))
        .await;

    assert_eq!(outcome, ReviewSpawnOutcome::ReviewRunning);
    let review = sqlx::query!(
        r#"SELECT task_title, task_slug, work_type as "work_type: WorkRunType",
           parent_work_run_id, review_target_pr_url, review_target_repo_full_name
           FROM work_runs
           WHERE parent_work_run_id = $1"#,
        run.id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should load spawned review run");
    assert_eq!(review.work_type, WorkRunType::PullRequestReview);
    assert_eq!(review.parent_work_run_id, Some(run.id));
    assert_eq!(
        review.task_title.as_deref(),
        Some("Fix inherited review metadata")
    );
    assert_eq!(review.task_slug.as_deref(), Some("VLC-409"));
    assert_eq!(
        review.review_target_pr_url.as_deref(),
        Some(pr_url.as_str())
    );
    assert_eq!(
        review.review_target_repo_full_name.as_deref(),
        Some("acme/app")
    );
}
