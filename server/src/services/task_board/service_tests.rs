use chrono::Utc;
use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::{
    IntegrationBoard, IntegrationBoardColumn, IntegrationColumn, IntegrationProject,
    IntegrationTask, IntegrationType,
};
use crate::services::task_board::service::{
    collect_board_task_refs, default_column_status, project_config_to_provider_project,
};

fn column(slug: &str, is_final: Option<bool>) -> IntegrationColumn {
    IntegrationColumn {
        id: slug.to_owned(),
        name: slug.to_owned(),
        slug: slug.to_owned(),
        is_final,
    }
}

fn board_column(slug: &str, tasks: Vec<IntegrationTask>) -> IntegrationBoardColumn {
    IntegrationBoardColumn {
        id: slug.to_owned(),
        name: slug.to_owned(),
        slug: slug.to_owned(),
        is_final: None,
        tasks,
    }
}

fn board(columns: Vec<IntegrationBoardColumn>) -> IntegrationBoard {
    IntegrationBoard {
        project: IntegrationProject {
            id: "project-1".to_owned(),
            name: "Project 1".to_owned(),
            slug: "project-1".to_owned(),
            workspace_id: Some("workspace-1".to_owned()),
        },
        columns,
        labels: Vec::new(),
    }
}

fn integration_task(id: &str) -> IntegrationTask {
    IntegrationTask {
        id: id.to_owned(),
        title: id.to_owned(),
        project_id: "project-1".to_owned(),
        description: None,
        status: "todo".to_owned(),
        priority: "medium".to_owned(),
        number: None,
        project_slug: Some("project-1".to_owned()),
        assignee_name: None,
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        updated_at: None,
        labels: Vec::new(),
    }
}

fn project_config(name: &str, provider_id: Option<Uuid>) -> ProjectConfig {
    ProjectConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        external_project_id: "external-1".to_owned(),
        name: name.to_owned(),
        external_workspace_id: "workspace-1".to_owned(),
        integration_type: IntegrationType::Kaneo,
        enabled: true,
        pickup_column: "to-do".to_owned(),
        target_column: "done".to_owned(),
        progress_column: "in-progress".to_owned(),
        max_turns: 3,
        prompt_template: None,
        repo_url: String::new(),
        repo_full_names: Vec::new(),
        repo_urls: Vec::new(),
        agents_md: None,
        review_enabled: None,
        review_max_turns: None,
        review_prompt_template: None,
        max_in_progress_tasks: None,
        created_at: Utc::now(),
        provider_id,
    }
}

#[test]
fn default_column_status_prefers_first_non_final_column() {
    let columns = vec![
        column("done", Some(true)),
        column("in-progress", Some(false)),
    ];

    assert_eq!(default_column_status(&columns), "in-progress");
}

#[test]
fn default_column_status_falls_back_when_provider_has_no_columns() {
    assert_eq!(default_column_status(&[]), "planned");
}

#[test]
fn project_config_mapping_uses_added_project_config() {
    let provider_id = Uuid::new_v4();
    let project = project_config("Added project", Some(provider_id));

    let result = project_config_to_provider_project(project).expect("project maps");

    assert_eq!(result.provider_id, provider_id);
    assert_eq!(result.external_project_id, "external-1");
    assert_eq!(result.workspace_id, "workspace-1");
    assert_eq!(result.name, "Added project");
}

#[test]
fn project_config_mapping_skips_legacy_configs_without_provider() {
    assert!(project_config_to_provider_project(project_config("", None)).is_none());
}

#[test]
fn collect_board_task_refs_deduplicates_in_board_order() {
    let board = board(vec![
        board_column(
            "todo",
            vec![integration_task("task-a"), integration_task("task-b")],
        ),
        board_column(
            "doing",
            vec![
                integration_task("task-b"),
                integration_task("task-c"),
                integration_task("task-a"),
            ],
        ),
    ]);

    assert_eq!(
        collect_board_task_refs(&board),
        vec![
            "task-a".to_owned(),
            "task-b".to_owned(),
            "task-c".to_owned(),
        ]
    );
}
