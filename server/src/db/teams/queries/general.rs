use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::teams::TeamsRepository;
use crate::models::teams::errors::TeamsError;
use crate::models::teams::model::{
    Team, TeamMember, DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE,
};

impl TeamsRepository {
    pub async fn create_personal_team<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
        name: &str,
    ) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Team,
            r#"INSERT INTO teams (id, name, personal_user_id, prompt_template, review_prompt_template)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
              primary_model_id, small_model_provider_key, small_model_id,
              review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
              created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            name,
            user_id,
            DEFAULT_PROMPT_TEMPLATE,
            DEFAULT_REVIEW_PROMPT_TEMPLATE,
        )
        .fetch_one(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn create_team<'c, Q>(&self, db: Q, name: &str) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Team,
            r#"INSERT INTO teams (id, name, prompt_template, review_prompt_template)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
              primary_model_id, small_model_provider_key, small_model_id,
              review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
              created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            name,
            DEFAULT_PROMPT_TEMPLATE,
            DEFAULT_REVIEW_PROMPT_TEMPLATE,
        )
        .fetch_one(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn get_by_id<'c, Q>(&self, db: Q, team_id: Uuid) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
             primary_model_id, small_model_provider_key, small_model_id,
             review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM teams WHERE id = $1"#,
            team_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(TeamsError::NotFound)
    }

    pub async fn lock_personal_team_creation<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<(), TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            user_id
        )
        .execute(db)
        .await
        .map(|_| ())
        .map_err(TeamsError::from)
    }

    pub async fn get_personal_team<'c, Q>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<Option<Team>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
             primary_model_id, small_model_provider_key, small_model_id,
             review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM teams WHERE personal_user_id = $1"#,
            user_id,
        )
        .fetch_optional(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn add_member<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
        role: &str,
    ) -> Result<TeamMember, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            TeamMember,
            r#"INSERT INTO team_members (team_id, user_id, role)
             VALUES ($1, $2, $3)
             ON CONFLICT (team_id, user_id) DO UPDATE SET role = EXCLUDED.role
             RETURNING team_id, user_id, role, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            user_id,
            role,
        )
        .fetch_one(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn add_member_preserving_owner<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
        role: &str,
    ) -> Result<TeamMember, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            TeamMember,
            r#"INSERT INTO team_members (team_id, user_id, role)
             VALUES ($1, $2, $3)
             ON CONFLICT (team_id, user_id) DO UPDATE SET role = CASE
                 WHEN team_members.role = 'owner' THEN team_members.role
                 ELSE EXCLUDED.role
             END
             RETURNING team_id, user_id, role, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            user_id,
            role,
        )
        .fetch_one(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn list_for_user<'c, Q>(&self, db: Q, user_id: &str) -> Result<Vec<Team>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT t.id, t.name, t.personal_user_id, t.prompt_template, t.agents_md, t.primary_model_provider_key,
             t.primary_model_id, t.small_model_provider_key, t.small_model_id,
             t.review_enabled, t.review_max_turns, t.review_prompt_template, t.max_in_progress_tasks, t.agent_backend,
             t.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM teams t
             INNER JOIN team_members tm ON tm.team_id = t.id
             WHERE tm.user_id = $1
             ORDER BY t.created_at ASC"#,
            user_id,
        )
        .fetch_all(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn list_all<'c, Q>(&self, db: Q) -> Result<Vec<Team>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
             primary_model_id, small_model_provider_key, small_model_id,
             review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM teams
             ORDER BY created_at ASC"#,
        )
        .fetch_all(db)
        .await
        .map_err(TeamsError::from)
    }

    pub async fn get_default_team<'c, Q>(&self, db: Q) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT id, name, personal_user_id, prompt_template, agents_md, primary_model_provider_key,
             primary_model_id, small_model_provider_key, small_model_id,
             review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks, agent_backend,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM teams ORDER BY created_at ASC LIMIT 1"#,
        )
        .fetch_optional(db)
        .await?
        .ok_or(TeamsError::NotFound)
    }

    pub async fn verify_membership<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
    ) -> Result<(), TeamsError>
    where
        Q: Queryer<'c>,
    {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            team_id,
            user_id,
        )
        .fetch_one(db)
        .await?
        .unwrap_or(false);

        match exists {
            true => Ok(()),
            false => Err(TeamsError::AccessDenied),
        }
    }

    pub async fn get_member_role<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
    ) -> Result<String, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!(
            "SELECT role FROM team_members WHERE team_id = $1 AND user_id = $2",
            team_id,
            user_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(TeamsError::AccessDenied)
    }
}
