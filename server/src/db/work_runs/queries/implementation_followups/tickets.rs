use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::queries::implementation_followups::FollowupTicketReservation;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

impl WorkRunsRepository {
    pub async fn reserve_github_implementation_followup_ticket<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        known_external_task_ref: Option<&str>,
        delivery_id: &str,
    ) -> Result<FollowupTicketReservation, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let token = Uuid::new_v4();
        let row = sqlx::query!(
            r#"WITH inserted AS (
                INSERT INTO github_implementation_followup_tickets
                    (project_config_id, repo_full_name, pr_number, external_task_ref,
                     created_by_delivery_id, operation_token, operation_delivery_id,
                     operation_started_at)
                VALUES ($1, $2, $3, $4,
                    CASE WHEN $4::TEXT IS NULL THEN $5 ELSE NULL END, $6, $5, NOW())
                ON CONFLICT DO NOTHING
                RETURNING external_task_ref, created_by_delivery_id, operation_token,
                    TRUE AS acquired
            ),
            acquired AS (
                UPDATE github_implementation_followup_tickets AS ticket
                SET external_task_ref = COALESCE(ticket.external_task_ref, $4),
                    created_by_delivery_id = CASE
                        WHEN ticket.external_task_ref IS NULL THEN $5
                        ELSE ticket.created_by_delivery_id
                    END,
                    operation_token = $6,
                    operation_delivery_id = $5,
                    operation_started_at = NOW()
                WHERE ticket.project_config_id = $1
                  AND ticket.repo_full_name = $2
                  AND ticket.pr_number = $3
                  AND (ticket.operation_token IS NULL
                       OR ticket.operation_started_at <= NOW() - INTERVAL '5 minutes')
                  AND NOT EXISTS (SELECT 1 FROM inserted)
                RETURNING ticket.external_task_ref, ticket.created_by_delivery_id,
                    ticket.operation_token, TRUE AS acquired
            ),
            current AS (
                SELECT external_task_ref, created_by_delivery_id, operation_token,
                    FALSE AS acquired
                FROM github_implementation_followup_tickets
                WHERE project_config_id = $1
                  AND repo_full_name = $2
                  AND pr_number = $3
                  AND NOT EXISTS (SELECT 1 FROM inserted)
                  AND NOT EXISTS (SELECT 1 FROM acquired)
            )
            SELECT external_task_ref, created_by_delivery_id,
                operation_token AS "operation_token?", acquired AS "acquired!"
            FROM inserted
            UNION ALL
            SELECT external_task_ref, created_by_delivery_id, operation_token, acquired FROM acquired
            UNION ALL
            SELECT external_task_ref, created_by_delivery_id, operation_token, acquired FROM current
            LIMIT 1"#,
            project_config_id,
            repo_full_name,
            pr_number,
            known_external_task_ref,
            delivery_id,
            token,
        )
        .fetch_optional(db)
        .await?;

        match row {
            Some(row) if row.acquired => match row.operation_token {
                Some(token) => Ok(FollowupTicketReservation::Acquired {
                    token,
                    external_task_ref: row.external_task_ref,
                    created_by_delivery_id: row.created_by_delivery_id,
                }),
                None => Ok(FollowupTicketReservation::Pending),
            },
            Some(_) | None => Ok(FollowupTicketReservation::Pending),
        }
    }

    pub async fn renew_github_implementation_followup_ticket<'c, Q>(
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
            r#"UPDATE github_implementation_followup_tickets
               SET operation_started_at = NOW()
               WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3
                 AND operation_token = $4"#,
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

    pub async fn complete_github_implementation_followup_ticket<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
        external_task_ref: &str,
    ) -> Result<(bool, Option<String>), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let row = sqlx::query!(
            r#"UPDATE github_implementation_followup_tickets
               SET external_task_ref = $5, operation_token = NULL,
                   operation_delivery_id = NULL, operation_started_at = NULL
               WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3
                 AND operation_token = $4
               RETURNING created_by_delivery_id"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
            external_task_ref,
        )
        .fetch_optional(db)
        .await?;

        Ok(match row {
            Some(row) => (true, row.created_by_delivery_id),
            None => (false, None),
        })
    }

    pub async fn release_github_implementation_followup_ticket<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            r#"UPDATE github_implementation_followup_tickets
               SET operation_token = NULL, operation_delivery_id = NULL,
                   operation_started_at = NULL
               WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3
                 AND operation_token = $4"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
