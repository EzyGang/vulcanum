use sqlx::PgPool;
use uuid::Uuid;

use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReviewTicketReservation {
    Ready(String),
    Acquired { token: Uuid, recovering: bool },
    Pending,
}

impl WorkRunsRepository {
    pub async fn reserve_github_review_ticket(
        &self,
        db: &PgPool,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<ReviewTicketReservation, WorkRunsError> {
        let token = Uuid::new_v4();
        let inserted = sqlx::query!(
            r#"INSERT INTO github_review_tickets
               (project_config_id, repo_full_name, pr_number, creation_token)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT DO NOTHING"#,
            project_config_id,
            repo_full_name,
            pr_number,
            token,
        )
        .execute(db)
        .await?
        .rows_affected()
            == 1;
        if inserted {
            return Ok(ReviewTicketReservation::Acquired {
                token,
                recovering: false,
            });
        }

        let current = self
            .github_review_ticket_state(db, project_config_id, repo_full_name, pr_number)
            .await?;
        match current.external_task_ref {
            Some(external_task_ref) => Ok(ReviewTicketReservation::Ready(external_task_ref)),
            None if !current.stale => Ok(ReviewTicketReservation::Pending),
            None => {
                let acquired = sqlx::query_scalar!(
                    r#"UPDATE github_review_tickets
                       SET creation_token = $4, creation_started_at = NOW()
                       WHERE project_config_id = $1
                         AND repo_full_name = $2
                         AND pr_number = $3
                         AND external_task_ref IS NULL
                         AND creation_started_at <= NOW() - INTERVAL '5 minutes'
                       RETURNING creation_token"#,
                    project_config_id,
                    repo_full_name,
                    pr_number,
                    token,
                )
                .fetch_optional(db)
                .await?;
                match acquired {
                    Some(token) => Ok(ReviewTicketReservation::Acquired {
                        token,
                        recovering: true,
                    }),
                    None => {
                        let current = self
                            .github_review_ticket_state(
                                db,
                                project_config_id,
                                repo_full_name,
                                pr_number,
                            )
                            .await?;
                        match current.external_task_ref {
                            Some(external_task_ref) => {
                                Ok(ReviewTicketReservation::Ready(external_task_ref))
                            }
                            None => Ok(ReviewTicketReservation::Pending),
                        }
                    }
                }
            }
        }
    }

    pub async fn finalize_github_review_ticket(
        &self,
        db: &PgPool,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
        external_task_ref: &str,
    ) -> Result<bool, WorkRunsError> {
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

    pub async fn renew_github_review_ticket_reservation(
        &self,
        db: &PgPool,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        token: Uuid,
    ) -> Result<bool, WorkRunsError> {
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

    async fn github_review_ticket_state(
        &self,
        db: &PgPool,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<ReviewTicketState, WorkRunsError> {
        let row = sqlx::query!(
            r#"SELECT external_task_ref,
                      creation_started_at <= NOW() - INTERVAL '5 minutes' AS "stale!"
               FROM github_review_tickets
               WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3"#,
            project_config_id,
            repo_full_name,
            pr_number,
        )
        .fetch_one(db)
        .await?;

        Ok(ReviewTicketState {
            external_task_ref: row.external_task_ref,
            stale: row.stale,
        })
    }
}

struct ReviewTicketState {
    external_task_ref: Option<String>,
    stale: bool,
}
