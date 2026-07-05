use uuid::Uuid;
use vulcanum_shared::api_types::{AgentBackend, JobRepo};

use crate::models::project_configs::model::JobConfigFields;
use crate::models::providers::model::IntegrationTask;
use crate::services::work_runs::service::get_job::render_implementation_prompt;

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
