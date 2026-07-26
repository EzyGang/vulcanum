use super::*;

pub async fn insert_project_config(pool: &sqlx::PgPool, external_project_id: &str) -> Uuid {
    teams::ensure_default_team(pool).await;
    insert_project_config_for_team(pool, DEFAULT_TEAM_ID, external_project_id).await
}

pub async fn insert_project_config_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    external_project_id: &str,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        teams::ensure_default_team(pool).await;
    }

    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, integration_type) VALUES ($1, $2, $3, 'kaneo')",
        id,
        team_id,
        external_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub async fn insert_project_config_with_provider(
    pool: &sqlx::PgPool,
    external_project_id: &str,
    provider_id: Uuid,
) -> Uuid {
    teams::ensure_default_team(pool).await;
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, integration_type, provider_id) VALUES ($1, $2, $3, 'kaneo', $4)",
        id,
        DEFAULT_TEAM_ID,
        external_project_id,
        provider_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}
