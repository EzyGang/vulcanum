use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api::wire::{AgentBackend, JobRepo};

use crate::models::project_configs::model::JobConfigFields;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::providers::client::TaskFetcher;
use crate::services::work_runs::service::get_job::render_implementation_prompt;
use crate::test_helpers;

#[test]
fn implementation_prompt_uses_work_run_repos_not_current_config() {
    let cfg = JobConfigFields {
        team_id: Uuid::new_v4(),
        external_project_id: "project-1".to_owned(),
        external_workspace_id: "workspace-1".to_owned(),
        max_turns: 1,
        review_max_turns: 1,
        provider_id: None,
        repo_urls: vec!["https://github.com/new/repo".to_owned()],
        repo_full_names: vec!["new/repo".to_owned()],
        prompt_template: "URL {{repo_url}}\nURLs {{repo_urls}}\nNames {{repo_names}}\nLayout {{repo_layout}}\nTask {{task_title}} {{task_body}}".to_owned(),
        agents_md: String::new(),
        review_prompt_template: String::new(),
        agent_backend: AgentBackend::default(),
    };
    let task = IntegrationTask {
        id: "task-1".to_owned(),
        title: "Fix checkout".to_owned(),
        project_id: "project-1".to_owned(),
        description: Some("Checkout fails on submit".to_owned()),
        status: "in-progress".to_owned(),
        priority: "medium".to_owned(),
        number: None,
        project_slug: None,
        assignee_name: None,
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        updated_at: None,
        labels: Vec::new(),
    };
    let repos = vec![JobRepo {
        full_name: "old/repo".to_owned(),
        url: "https://github.com/old/repo".to_owned(),
    }];

    let prompt = render_implementation_prompt(&cfg, &task, &repos);

    assert!(prompt.contains("https://github.com/old/repo"));
    assert!(prompt.contains("old/repo"));
    assert!(prompt.contains("old/repo: ./old-repo"));
    assert!(!prompt.contains("https://github.com/new/repo"));
    assert!(!prompt.contains("new/repo"));
}

#[test]
fn parentless_linked_review_is_not_standalone() {
    let mut run = standalone_review();
    run.github_delivery_id = None;

    assert!(!run.is_standalone_review());
}

#[sqlx::test]
async fn standalone_review_token_uses_triggering_installation(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let mut run = standalone_review();
    run.team_id = test_helpers::DEFAULT_TEAM_ID;
    let repos = vec![JobRepo {
        full_name: "acme/widgets".to_owned(),
        url: "https://github.com/acme/widgets".to_owned(),
    }];

    let error = state
        .jobs
        .mint_github_token_for_repos(&run, &repos)
        .await
        .expect_err("disconnected triggering installation must be rejected");

    assert!(matches!(
        error,
        crate::models::work_runs::errors::WorkRunsError::GithubApp(
            crate::models::github_app::errors::GithubAppError::NoInstallation
        )
    ));
}

#[sqlx::test]
async fn standalone_review_bypasses_task_fetcher(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool).await;
    let service = state.jobs.with_task_fetcher(Arc::new(FailingTaskFetcher));
    let run = standalone_review();
    let cfg = JobConfigFields::empty_for_team(run.team_id);

    let task = service
        .fetch_task_for_run(&run, &cfg)
        .await
        .expect("standalone task fallback");

    assert_eq!(task.title, "Review pull request");
    assert_eq!(task.description, None);
}

struct FailingTaskFetcher;

#[async_trait]
impl TaskFetcher for FailingTaskFetcher {
    async fn fetch_tasks_in_column(
        &self,
        _project_id: &str,
        _column_name: &str,
    ) -> Result<Vec<IntegrationTask>, IntegrationError> {
        Err(IntegrationError::Other("must not fetch tasks".to_owned()))
    }

    async fn fetch_task(&self, _task_id: &str) -> Result<IntegrationTask, IntegrationError> {
        Err(IntegrationError::Other("must not fetch task".to_owned()))
    }

    async fn update_task_status(
        &self,
        _task_id: &str,
        _new_status: &str,
    ) -> Result<(), IntegrationError> {
        Err(IntegrationError::Other(
            "must not update task status".to_owned(),
        ))
    }
}

fn standalone_review() -> WorkRun {
    let now = Utc::now();
    WorkRun {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        external_task_ref: "github-pr:acme/widgets#42".to_owned(),
        task_title: Some("Review pull request".to_owned()),
        task_slug: Some("acme/widgets#42".to_owned()),
        project_config_id: Uuid::new_v4(),
        worker_id: None,
        status: WorkRunStatus::Pending,
        work_type: WorkRunType::PullRequestReview,
        parent_work_run_id: None,
        review_target_pr_url: Some("https://github.com/acme/widgets/pull/42".to_owned()),
        review_target_repo_full_name: Some("acme/widgets".to_owned()),
        github_installation_id: Some(123),
        github_delivery_id: Some("delivery-1".to_owned()),
        result_pr_url: None,
        result_exit_code: None,
        tokens_used: None,
        duration_ms: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        model_used: None,
        finish_status: None,
        result_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
        created_at: now,
        updated_at: now,
    }
}
