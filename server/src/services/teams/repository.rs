use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::model::{Team, TeamMember, UserIdentity};

#[derive(Clone, Default)]
pub struct TeamsRepository;

impl TeamsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

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
            r#"INSERT INTO teams (id, name, personal_user_id)
             VALUES ($1, $2, $3)
             RETURNING id, name, personal_user_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            id,
            name,
            user_id,
        )
        .fetch_one(db)
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

    pub async fn list_for_user<'c, Q>(&self, db: Q, user_id: &str) -> Result<Vec<Team>, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT t.id, t.name, t.personal_user_id, t.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
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

    pub async fn get_default_team<'c, Q>(&self, db: Q) -> Result<Team, TeamsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            Team,
            r#"SELECT id, name, personal_user_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
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
        sqlx::query!(
            r#"INSERT INTO user_identities (id, user_id, provider, provider_user_id, provider_login)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (provider, provider_user_id) DO UPDATE SET
                user_id = EXCLUDED.user_id,
                provider_login = EXCLUDED.provider_login,
                updated_at = NOW()"#,
            id,
            user_id,
            provider,
            provider_user_id,
            provider_login,
        )
        .execute(db)
        .await
        .map(|_| ())
        .map_err(TeamsError::from)
    }
}
