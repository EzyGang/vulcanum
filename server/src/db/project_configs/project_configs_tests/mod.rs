mod crud;
mod lookup;

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::project_configs::{ProjectConfigsRepository, UpdateProjectConfigParams};
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::CreateProjectConfigRequest;
use crate::models::providers::model::IntegrationType;
use crate::test_helpers::DEFAULT_TEAM_ID;

async fn insert_provider(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::teams::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        id,
        DEFAULT_TEAM_ID,
        "test-provider",
        "cloud.kaneo.app",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("Should insert provider");

    id
}

async fn insert_provider_named(pool: &PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::teams::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        id,
        DEFAULT_TEAM_ID,
        name,
        "cloud.kaneo.app",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("Should insert named provider");

    id
}

fn test_params(external_project_id: &str, provider_id: Uuid) -> CreateProjectConfigRequest {
    CreateProjectConfigRequest {
        external_project_id: external_project_id.to_owned(),
        name: String::new(),
        enabled: true,
        pickup_column: "to-do".to_owned(),
        progress_column: "in-progress".to_owned(),
        review_column: "in-review".to_owned(),
        done_column: "done".to_owned(),
        prompt_template: Some("Review {{task_title}}".to_owned()),
        repo_full_names: Vec::new(),
        agents_md: Some(String::new()),
        review_enabled: None,
        review_max_turns: None,
        review_prompt_template: None,
        max_in_progress_tasks: None,
        max_turns: 3,
        external_workspace_id: String::new(),
        integration_type: IntegrationType::Kaneo,
        provider_id,
    }
}

fn test_update_params() -> UpdateProjectConfigParams<'static> {
    UpdateProjectConfigParams {
        name: None,
        pickup_column: None,
        review_column: None,
        done_column: None,
        progress_column: None,
        max_turns: None,
        prompt_template: None,
        repo_url: None,
        agents_md: None,
        review_enabled: None,
        review_max_turns: None,
        review_prompt_template: None,
        max_in_progress_tasks: None,
        external_workspace_id: None,
        enabled: None,
        integration_type: None,
        provider_id: None,
    }
}
