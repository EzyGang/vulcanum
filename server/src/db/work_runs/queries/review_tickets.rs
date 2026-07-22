use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReviewTicketReservation {
    Ready(String),
    Acquired { token: Uuid, recovering: bool },
    Pending,
}

impl WorkRunsRepository {
    pub async fn reserve_github_review_ticket<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<ReviewTicketReservation, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let token = Uuid::new_v4();
        let reservation = sqlx::query!(
            r#"WITH inserted AS (
                   INSERT INTO github_review_tickets
                       (project_config_id, repo_full_name, pr_number, creation_token)
                   VALUES ($1, $2, $3, $4)
                   ON CONFLICT DO NOTHING
                   RETURNING external_task_ref, creation_token,
                       FALSE AS recovering, TRUE AS acquired
               ),
               acquired AS (
                   UPDATE github_review_tickets
                   SET creation_token = $4, creation_started_at = NOW()
                   WHERE project_config_id = $1
                     AND repo_full_name = $2
                     AND pr_number = $3
                     AND external_task_ref IS NULL
                     AND creation_started_at <= NOW() - INTERVAL '5 minutes'
                     AND NOT EXISTS (SELECT 1 FROM inserted)
                   RETURNING external_task_ref, creation_token,
                       TRUE AS recovering, TRUE AS acquired
               ),
               current AS (
                   SELECT external_task_ref, creation_token,
                       FALSE AS recovering, FALSE AS acquired
                   FROM github_review_tickets
                   WHERE project_config_id = $1
                     AND repo_full_name = $2
                     AND pr_number = $3
                     AND NOT EXISTS (SELECT 1 FROM inserted)
                     AND NOT EXISTS (SELECT 1 FROM acquired)
               )
               SELECT external_task_ref,
                   creation_token AS "creation_token!",
                   recovering AS "recovering!",
                   acquired AS "acquired!"
               FROM inserted
               UNION ALL
               SELECT external_task_ref, creation_token, recovering, acquired
               FROM acquired
               UNION ALL
               SELECT external_task_ref, creation_token, recovering, acquired
               FROM current
               LIMIT 1"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
        )
        .fetch_optional(db)
        .await?;

        match reservation {
            Some(row) => match row.external_task_ref {
                Some(external_task_ref) => Ok(ReviewTicketReservation::Ready(external_task_ref)),
                None if row.acquired => Ok(ReviewTicketReservation::Acquired {
                    token: row.creation_token,
                    recovering: row.recovering,
                }),
                None => Ok(ReviewTicketReservation::Pending),
            },
            None => Ok(ReviewTicketReservation::Pending),
        }
    }

    pub async fn finalize_github_review_ticket<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
        external_task_ref: &str,
    ) -> Result<bool, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let updated = sqlx::query!(
            r#"UPDATE github_review_tickets
               SET external_task_ref = $5
               WHERE project_config_id = $1
                 AND repo_full_name = $2
                 AND pr_number = $3
                 AND creation_token = $4
                 AND external_task_ref IS NULL"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
            external_task_ref,
        )
        .execute(db)
        .await?
        .rows_affected()
            == 1;

        Ok(updated)
    }

    pub async fn renew_github_review_ticket_reservation<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
    ) -> Result<bool, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let renewed = sqlx::query!(
            r#"UPDATE github_review_tickets
               SET creation_started_at = NOW()
               WHERE project_config_id = $1
                 AND repo_full_name = $2
                 AND pr_number = $3
                 AND creation_token = $4
                 AND external_task_ref IS NULL"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
        )
        .execute(db)
        .await?
        .rows_affected()
            == 1;

        Ok(renewed)
    }
}
