use uuid::Uuid;

use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::queryer::Queryer;
use crate::models::project_configs::errors::ProjectConfigsError;

impl ProjectConfigsRepository {
    pub async fn count_enabled<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<i64, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM project_configs WHERE enabled = true AND team_id = $1",
            team_id,
        )
        .fetch_one(db)
        .await?
        .unwrap_or(0);

        Ok(count)
    }
}
