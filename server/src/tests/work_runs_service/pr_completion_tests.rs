use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::db::work_runs::WorkRunsRepository;
use crate::models::github_app::errors::GithubAppError;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::TaskPr;
use crate::services::github_app::service::pull_requests::{
    PullRequestState, PullRequestStateReader,
};
use crate::test_helpers;

struct FakePullRequestStateReader {
    states: HashMap<i64, Option<PullRequestState>>,
}

#[async_trait]
impl PullRequestStateReader for FakePullRequestStateReader {
    async fn pull_request_state(
        &self,
        _team_id: Uuid,
        _repo_full_name: &str,
        number: i64,
    ) -> Result<PullRequestState, GithubAppError> {
        match self.states.get(&number).copied().flatten() {
            Some(state) => Ok(state),
            None => Err(GithubAppError::Api("state unavailable".to_owned())),
        }
    }
}

#[sqlx::test]
async fn terminal_pr_set_requires_at_least_one_pr(pool: sqlx::PgPool) {
    let service = test_helpers::build_state(pool)
        .await
        .jobs
        .with_pr_state_reader(Arc::new(FakePullRequestStateReader {
            states: HashMap::new(),
        }));

    assert!(!service
        .task_prs_are_terminal(Uuid::new_v4(), "task-1", &[])
        .await
        .expect("evaluate empty PR set"));
}

#[sqlx::test]
async fn closed_and_merged_prs_are_terminal(pool: sqlx::PgPool) {
    let service = test_helpers::build_state(pool)
        .await
        .jobs
        .with_pr_state_reader(Arc::new(FakePullRequestStateReader {
            states: HashMap::from([
                (1, Some(PullRequestState::Closed)),
                (2, Some(PullRequestState::Merged)),
            ]),
        }));
    let task_prs = vec![task_pr(1), task_pr(2)];

    assert!(service
        .task_prs_are_terminal(Uuid::new_v4(), "task-1", &task_prs)
        .await
        .expect("evaluate terminal PR set"));
}

#[sqlx::test]
async fn open_pr_blocks_completion_and_lookup_failure_retries(pool: sqlx::PgPool) {
    let open_service = test_helpers::build_state(pool.clone())
        .await
        .jobs
        .with_pr_state_reader(Arc::new(FakePullRequestStateReader {
            states: HashMap::from([
                (1, Some(PullRequestState::Closed)),
                (2, Some(PullRequestState::Open)),
            ]),
        }));
    let failed_service = test_helpers::build_state(pool)
        .await
        .jobs
        .with_pr_state_reader(Arc::new(FakePullRequestStateReader {
            states: HashMap::from([(1, Some(PullRequestState::Closed)), (2, None)]),
        }));
    let task_prs = vec![task_pr(1), task_pr(2)];

    assert!(!open_service
        .task_prs_are_terminal(Uuid::new_v4(), "task-1", &task_prs)
        .await
        .expect("evaluate open PR set"));
    assert!(matches!(
        failed_service
            .task_prs_are_terminal(Uuid::new_v4(), "task-1", &task_prs)
            .await,
        Err(WorkRunsError::GithubApp(_))
    ));
}

#[sqlx::test]
async fn webhook_target_lookup_is_scoped_to_installation_and_pr(pool: sqlx::PgPool) {
    let project_config_id = test_helpers::insert_project_config(&pool, "webhook-target").await;
    sqlx::query!(
        "INSERT INTO github_installations \
         (github_installation_id, account_login, team_id) VALUES ($1, $2, $3)",
        123_i64,
        "acme",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(&pool)
    .await
    .expect("insert GitHub installation");
    sqlx::query!(
        "INSERT INTO task_prs \
         (project_config_id, external_task_ref, pr_url, repo_full_name, pr_number) \
         VALUES ($1, $2, $3, $4, $5)",
        project_config_id,
        "task-1",
        "https://github.com/acme/app/pull/42",
        "acme/app",
        42_i64,
    )
    .execute(&pool)
    .await
    .expect("insert task PR");
    let repository = WorkRunsRepository::new();

    let targets = repository
        .list_task_pr_targets_for_pull_request(&pool, 123, "ACME/APP", 42)
        .await
        .expect("find webhook targets");
    let wrong_installation = repository
        .list_task_pr_targets_for_pull_request(&pool, 456, "acme/app", 42)
        .await
        .expect("scope webhook targets");

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].project_config_id, project_config_id);
    assert_eq!(targets[0].external_task_ref, "task-1");
    assert!(wrong_installation.is_empty());
}

fn task_pr(number: i64) -> TaskPr {
    let now = Utc::now();
    TaskPr {
        id: Uuid::new_v4(),
        project_config_id: Uuid::new_v4(),
        external_task_ref: "task-1".to_owned(),
        pr_url: format!("https://github.com/acme/app/pull/{number}"),
        repo_full_name: "acme/app".to_owned(),
        pr_number: number,
        source_work_run_id: None,
        created_at: now,
        updated_at: now,
    }
}
