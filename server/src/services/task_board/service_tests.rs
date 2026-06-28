use chrono::Utc;
use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::IntegrationColumn;
use crate::models::providers::model::IntegrationType;
use crate::services::task_board::service::{
    default_column_status, project_config_to_provider_project,
};

fn column(slug: &str, is_final: Option<bool>) -> IntegrationColumn {
    IntegrationColumn {
        id: slug.to_owned(),
        name: slug.to_owned(),
        slug: slug.to_owned(),
        is_final,
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
        primary_model_provider_key: None,
        primary_model_id: None,
        small_model_provider_key: None,
        small_model_id: None,
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
