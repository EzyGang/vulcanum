use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::util::github::github_repo_url;

impl WorkRunsRepository {
    pub async fn insert_work_run<'c, Q>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<WorkRun, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        let (repo_urls, positions) = repo_insert_arrays(&params.repo_full_names);

        sqlx::query_as!(
            WorkRun,
            r#"WITH inserted AS (
                INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, task_title, task_slug, status,
                 work_type, parent_work_run_id, review_target_pr_url, review_target_repo_full_name,
                 github_installation_id, github_delivery_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id, team_id, external_task_ref, task_title, task_slug, project_config_id, worker_id, status,
                 work_type, parent_work_run_id, review_target_pr_url, review_target_repo_full_name,
                 github_installation_id, github_delivery_id, result_pr_url, result_exit_code, tokens_used,
                 duration_ms, input_tokens, output_tokens,
                 cache_read_tokens, cache_write_tokens, model_used, finish_status, result_summary,
                 finish_blocked_reason, finish_next_column, created_at, updated_at
            ),
            inserted_repos AS (
                INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
                SELECT inserted.id, repos.repo_full_name, repos.repo_url, repos.position
                FROM inserted
                JOIN UNNEST($14::text[], $15::text[], $16::int4[]) AS repos(repo_full_name, repo_url, position) ON TRUE
                RETURNING 1
            )
            SELECT id, team_id, external_task_ref, task_title as "task_title?: String", task_slug as "task_slug?: String", project_config_id, worker_id, status as "status: WorkRunStatus",
             work_type as "work_type: WorkRunType", parent_work_run_id,
             review_target_pr_url, review_target_repo_full_name,
             github_installation_id, github_delivery_id,
             result_pr_url, result_exit_code, tokens_used, duration_ms,
             input_tokens as "input_tokens?: i64", output_tokens as "output_tokens?: i64",
             cache_read_tokens as "cache_read_tokens?: i64", cache_write_tokens as "cache_write_tokens?: i64",
             model_used,
             finish_status, result_summary, finish_blocked_reason, finish_next_column,
             created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
             FROM inserted
             CROSS JOIN (SELECT COUNT(*) FROM inserted_repos) AS inserted_repo_count"#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            params.task_title.as_deref(),
            params.task_slug.as_deref(),
            &params.status as &WorkRunStatus,
            &params.work_type as &WorkRunType,
            params.parent_work_run_id,
            params.review_target_pr_url.as_deref(),
            params.review_target_repo_full_name.as_deref(),
            params.github_installation_id,
            params.github_delivery_id.as_deref(),
            &params.repo_full_names,
            &repo_urls,
            &positions,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn insert_work_run_if_not_active<'c, Q>(
        &self,
        db: Q,
        params: InsertWorkRunParams,
    ) -> Result<bool, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();
        let (repo_urls, positions) = repo_insert_arrays(&params.repo_full_names);

        let result = sqlx::query!(
            r#"WITH inserted AS (
                INSERT INTO work_runs (id, team_id, external_task_ref, project_config_id, task_title, task_slug, status,
                 work_type, parent_work_run_id, review_target_pr_url, review_target_repo_full_name,
                 github_installation_id, github_delivery_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT DO NOTHING
                RETURNING id
            ),
            inserted_repos AS (
                INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
                SELECT inserted.id, repos.repo_full_name, repos.repo_url, repos.position
                FROM inserted
                JOIN UNNEST($14::text[], $15::text[], $16::int4[]) AS repos(repo_full_name, repo_url, position) ON TRUE
                RETURNING 1
            )
            SELECT EXISTS(SELECT 1 FROM inserted) AS "inserted!"
            FROM (SELECT COUNT(*) FROM inserted_repos) AS inserted_repo_count"#,
            id,
            params.team_id,
            &params.external_task_ref,
            params.project_config_id,
            params.task_title.as_deref(),
            params.task_slug.as_deref(),
            &params.status as &WorkRunStatus,
            &params.work_type as &WorkRunType,
            params.parent_work_run_id,
            params.review_target_pr_url.as_deref(),
            params.review_target_repo_full_name.as_deref(),
            params.github_installation_id,
            params.github_delivery_id.as_deref(),
            &params.repo_full_names,
            &repo_urls,
            &positions,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(result.inserted)
    }
}

fn repo_insert_arrays(repo_full_names: &[String]) -> (Vec<String>, Vec<i32>) {
    let repo_urls = repo_full_names
        .iter()
        .map(|name| github_repo_url(name))
        .collect::<Vec<String>>();
    let positions = (0..repo_full_names.len() as i32).collect::<Vec<i32>>();

    (repo_urls, positions)
}
