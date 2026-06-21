DROP TRIGGER IF EXISTS trg_task_prs_updated_at ON task_prs;

DROP INDEX IF EXISTS unique_active_review_run_per_task_pr;
DROP INDEX IF EXISTS unique_active_implementation_task;

CREATE UNIQUE INDEX unique_active_task
    ON work_runs(project_config_id, external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running');

DROP TABLE IF EXISTS work_run_reviews;
DROP TABLE IF EXISTS task_prs;

ALTER TABLE work_runs
    DROP COLUMN review_already_exists,
    DROP COLUMN review_body,
    DROP COLUMN review_url,
    DROP COLUMN review_target_repo_full_name,
    DROP COLUMN review_target_pr_url,
    DROP COLUMN task_body,
    DROP COLUMN parent_work_run_id,
    DROP COLUMN work_type;

ALTER TABLE project_configs
    DROP COLUMN review_prompt_template,
    DROP COLUMN review_max_turns,
    DROP COLUMN review_pickup_column,
    DROP COLUMN review_enabled;

ALTER TABLE teams
    DROP COLUMN review_prompt_template,
    DROP COLUMN review_max_turns,
    DROP COLUMN review_pickup_column,
    DROP COLUMN review_enabled;

DROP TYPE work_run_type;
