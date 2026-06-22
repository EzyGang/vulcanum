use sqlx::Row;
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::model::{ProviderIdentity, Team, TeamMemberInfo, UserIdentity};
use crate::services::teams::repository::TeamsRepository;

impl TeamsRepository {
    pub async fn list_members<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<TeamMemberInfo>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            TeamMemberInfo,
            r#"SELECT tm.team_id, tm.user_id, u.email, tm.role, tm.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM team_members tm
             INNER JOIN users u ON u.id = tm.user_id
             WHERE tm.team_id = $1
             ORDER BY tm.created_at ASC"#,
            team_id,
        )
        .fetch_all(db)
        .await
        .map_err(TeamsError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_settings<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        name: Option<&str>,
        prompt_template: Option<&str>,
        agents_md: Option<&str>,
        primary_model_provider_key: Option<Option<&str>>,
        primary_model_provider_config_id: Option<Option<Uuid>>,
        primary_model_id: Option<Option<&str>>,
        small_model_provider_key: Option<Option<&str>>,
        small_model_provider_config_id: Option<Option<Uuid>>,
        small_model_id: Option<Option<&str>>,
        review_enabled: Option<bool>,
        review_pickup_column: Option<&str>,
        review_max_turns: Option<i32>,
        review_prompt_template: Option<&str>,
        max_in_progress_tasks: Option<i32>,
    ) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"UPDATE teams SET
             name = COALESCE($2, name),
             prompt_template = COALESCE($3, prompt_template),
             agents_md = COALESCE($4, agents_md),
             primary_model_provider_key = CASE WHEN $5 THEN $6 ELSE primary_model_provider_key END,
             primary_model_provider_config_id = CASE WHEN $7 THEN $8 ELSE primary_model_provider_config_id END,
             primary_model_id = CASE WHEN $9 THEN $10 ELSE primary_model_id END,
             small_model_provider_key = CASE WHEN $11 THEN $12 ELSE small_model_provider_key END,
             small_model_provider_config_id = CASE WHEN $13 THEN $14 ELSE small_model_provider_config_id END,
             small_model_id = CASE WHEN $15 THEN $16 ELSE small_model_id END,
             review_enabled = COALESCE($17, review_enabled),
             review_pickup_column = COALESCE($18, review_pickup_column),
             review_max_turns = COALESCE($19, review_max_turns),
             review_prompt_template = COALESCE($20, review_prompt_template),
             max_in_progress_tasks = COALESCE($21, max_in_progress_tasks)
             WHERE id = $1
             RETURNING id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
              primary_model_provider_config_id, primary_model_id, small_model_provider_key,
              small_model_provider_config_id, small_model_id,
              review_enabled, review_pickup_column, review_max_turns, review_prompt_template, max_in_progress_tasks,
              created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            name,
            prompt_template,
            agents_md,
            primary_model_provider_key.is_some(),
            primary_model_provider_key.flatten(),
            primary_model_provider_config_id.is_some(),
            primary_model_provider_config_id.flatten(),
            primary_model_id.is_some(),
            primary_model_id.flatten(),
            small_model_provider_key.is_some(),
            small_model_provider_key.flatten(),
            small_model_provider_config_id.is_some(),
            small_model_provider_config_id.flatten(),
            small_model_id.is_some(),
            small_model_id.flatten(),
            review_enabled,
            review_pickup_column,
            review_max_turns,
            review_prompt_template,
            max_in_progress_tasks,
        )
        .fetch_optional(db)
        .await?
        .ok_or(TeamsError::NotFound)
    }

    pub async fn delete<'c, Q>(&self, db: Q, team_id: Uuid) -> Result<(), TeamsError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!("DELETE FROM teams WHERE id = $1", team_id)
            .execute(db)
            .await?
            .rows_affected();

        match rows {
            0 => Err(TeamsError::NotFound),
            _ => Ok(()),
        }
    }

    pub async fn count_all<'c, Q>(&self, db: Q) -> Result<i64, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!("SELECT COUNT(*) as \"count!: i64\" FROM teams")
            .fetch_one(db)
            .await
            .map_err(TeamsError::from)
    }

    pub async fn count_for_user<'c, Q>(&self, db: Q, user_id: &str) -> Result<i64, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64"
             FROM teams t
             INNER JOIN team_members tm ON tm.team_id = t.id
             WHERE tm.user_id = $1"#,
            user_id,
        )
        .fetch_one(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn find_identity<'c, Q>(
        &self,
        db: Q,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<UserIdentity>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            UserIdentity,
            r#"SELECT user_id, provider_user_id, provider_login
             FROM user_identities WHERE provider = $1 AND provider_user_id = $2"#,
            provider,
            provider_user_id,
        )
        .fetch_optional(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn upsert_identity<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
        provider: &str,
        provider_user_id: &str,
        provider_login: &str,
    ) -> Result<(), TeamsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_login, provider_verified_at)
             VALUES ($1, $2, $3, $4, $5, NOW())
             ON CONFLICT (provider, provider_user_id) DO UPDATE SET
                 user_id = EXCLUDED.user_id,
                 provider_login = EXCLUDED.provider_login,
                 provider_verified_at = NOW(),
                 updated_at = NOW()"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(provider_login)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(TeamsError::from)
    }

    pub async fn list_identities_for_user<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<Vec<ProviderIdentity>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query(
            r#"SELECT provider, provider_user_id, provider_login, provider_verified_at
             FROM user_identities
             WHERE user_id = $1
             ORDER BY provider ASC"#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ProviderIdentity {
                    provider: row.try_get("provider")?,
                    provider_user_id: row.try_get("provider_user_id")?,
                    provider_login: row.try_get("provider_login")?,
                    provider_verified_at: row.try_get("provider_verified_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(TeamsError::from)
    }

    pub async fn user_has_identity<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
        provider: &str,
    ) -> Result<bool, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM user_identities WHERE user_id = $1 AND provider = $2
            )"#,
            user_id,
            provider,
        )
        .fetch_one(db)
        .await
        .map(|exists| exists.unwrap_or(false))
        .map_err(TeamsError::from)
    }
}
