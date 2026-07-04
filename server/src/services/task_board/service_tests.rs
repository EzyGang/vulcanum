use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::{
    IntegrationBoard, IntegrationBoardColumn, IntegrationColumn, IntegrationProject,
    IntegrationTask, IntegrationType,
};
use crate::models::work_runs::model::{TaskBoardRelatedWorkRunRow, WorkRunStatus, WorkRunType};
use crate::services::task_board::service::{
    collect_board_task_refs, default_column_status, group_related_runs,
    project_config_to_provider_project,
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

fn related_row(
    task_ref: &str,
    id: Uuid,
    model_used: &str,
    created_at: DateTime<Utc>,
) -> TaskBoardRelatedWorkRunRow {
    TaskBoardRelatedWorkRunRow {
        external_task_ref: task_ref.to_owned(),
        id,
        status: WorkRunStatus::Completed,
        work_type: WorkRunType::Implementation,
        tokens_used: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        model_used: Some(model_used.to_owned()),
        created_at,
    }
}

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("timestamp should parse")
        .with_timezone(&Utc)
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

#[test]
fn group_related_runs_preserves_board_task_order_and_run_order() {
    let task_b_run_id = Uuid::from_u128(11);
    let task_a_newer_run_id = Uuid::from_u128(21);
    let task_a_older_run_id = Uuid::from_u128(22);
    let unrelated_run_id = Uuid::from_u128(99);

    let grouped = group_related_runs(
        vec![
            "task-b".to_owned(),
            "task-a".to_owned(),
            "task-c".to_owned(),
        ],
        vec![
            related_row(
                "task-a",
                task_a_newer_run_id,
                "task-a-newer",
                timestamp("2026-01-01T00:02:00Z"),
            ),
            related_row(
                "task-b",
                task_b_run_id,
                "task-b-only",
                timestamp("2026-01-01T00:03:00Z"),
            ),
            related_row(
                "task-a",
                task_a_older_run_id,
                "task-a-older",
                timestamp("2026-01-01T00:01:00Z"),
            ),
            related_row(
                "task-unrequested",
                unrelated_run_id,
                "unrequested",
                timestamp("2026-01-01T00:04:00Z"),
            ),
        ],
    );

    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].external_task_ref, "task-b");
    assert_eq!(grouped[0].runs[0].id, task_b_run_id);
    assert_eq!(
        grouped[0].runs[0].model_used.as_deref(),
        Some("task-b-only")
    );
    assert_eq!(grouped[1].external_task_ref, "task-a");
    assert_eq!(
        grouped[1].runs.iter().map(|run| run.id).collect::<Vec<_>>(),
        vec![task_a_newer_run_id, task_a_older_run_id]
    );
    assert_eq!(
        grouped[1]
            .runs
            .iter()
            .map(|run| run.model_used.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("task-a-newer"), Some("task-a-older")]
    );
}
