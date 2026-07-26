use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::queries::implementation_followups::InsertFollowupRequestParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{
    GithubImplementationFollowupContext, GithubImplementationFollowupRequest,
};

impl WorkRunsRepository {
    pub async fn insert_or_get_github_implementation_followup<'c, Q>(
        &self,
        db: Q,
        params: InsertFollowupRequestParams<'_>,
    ) -> Result<GithubImplementationFollowupRequest, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            GithubImplementationFollowupRequest,
            r#"WITH inserted AS (
                INSERT INTO github_implementation_followup_requests
                    (delivery_id, github_installation_id, repo_full_name, pr_number, comment_id,
                     sender_id, project_config_id, request_body)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT DO NOTHING
                RETURNING delivery_id, github_installation_id, repo_full_name, pr_number, comment_id,
                    sender_id, project_config_id, external_task_ref, work_run_id, request_body,
                    ticket_created, outcome
            ),
            selected AS (
                SELECT delivery_id, github_installation_id, repo_full_name, pr_number, comment_id,
                    sender_id, project_config_id, external_task_ref, work_run_id, request_body,
                    ticket_created, outcome
                FROM inserted
                UNION ALL
                SELECT delivery_id, github_installation_id, repo_full_name, pr_number, comment_id,
                    sender_id, project_config_id, external_task_ref, work_run_id, request_body,
                    ticket_created, outcome
                FROM github_implementation_followup_requests
                WHERE delivery_id = $1 AND NOT EXISTS (SELECT 1 FROM inserted)
            )
            SELECT delivery_id AS "delivery_id!", github_installation_id AS "github_installation_id!",
                repo_full_name AS "repo_full_name!", pr_number AS "pr_number!",
                comment_id AS "comment_id!", sender_id AS "sender_id!",
                project_config_id AS "project_config_id!", external_task_ref, work_run_id,
                request_body AS "request_body!", ticket_created AS "ticket_created!",
                outcome AS "outcome!"
            FROM selected
            LIMIT 1"#,
            params.delivery_id,
            params.github_installation_id,
            params.repo_full_name,
            params.pr_number,
            params.comment_id,
            params.sender_id,
            params.project_config_id,
            params.request_body,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn finish_github_implementation_followup<'c, Q>(
        &self,
        db: Q,
        delivery_id: &str,
        external_task_ref: Option<&str>,
        work_run_id: Option<Uuid>,
        ticket_created: bool,
        outcome: &str,
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            r#"UPDATE github_implementation_followup_requests
               SET external_task_ref = COALESCE($2, external_task_ref),
                   work_run_id = COALESCE($3, work_run_id), ticket_created = $4, outcome = $5
               WHERE delivery_id = $1"#,
            delivery_id,
            external_task_ref,
            work_run_id,
            ticket_created,
            outcome,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn find_github_implementation_followup_context<'c, Q>(
        &self,
        db: Q,
        work_run_id: Uuid,
    ) -> Result<Option<GithubImplementationFollowupContext>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            GithubImplementationFollowupContext,
            r#"SELECT delivery_id, repo_full_name, pr_number, request_body
               FROM github_implementation_followup_requests
               WHERE work_run_id = $1"#,
            work_run_id,
        )
        .fetch_optional(db)
        .await
        .map_err(WorkRunsError::from)
    }
}
