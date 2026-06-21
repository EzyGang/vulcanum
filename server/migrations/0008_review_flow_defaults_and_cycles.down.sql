DROP INDEX IF EXISTS unique_active_review_run_per_task_pr;

CREATE UNIQUE INDEX IF NOT EXISTS unique_review_run_per_task_pr
    ON work_runs(project_config_id, external_task_ref, review_target_pr_url)
    WHERE work_type = 'pull_request_review'
      AND review_target_pr_url IS NOT NULL;
