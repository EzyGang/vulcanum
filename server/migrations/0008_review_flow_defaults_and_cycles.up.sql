ALTER TABLE teams
    ALTER COLUMN review_prompt_template DROP DEFAULT;

UPDATE teams
SET review_prompt_template = replace(review_prompt_template, E'\r\n', E'\n')
WHERE review_prompt_template LIKE '%' || E'\r\n' || '%';

UPDATE teams
SET review_prompt_template = ''
WHERE review_prompt_template LIKE '%{{review_marker}}%';

UPDATE project_configs
SET review_prompt_template = NULL
WHERE review_prompt_template LIKE '%{{review_marker}}%';

DROP INDEX IF EXISTS unique_review_run_per_task_pr;

CREATE UNIQUE INDEX IF NOT EXISTS unique_active_review_run_per_task_pr
    ON work_runs(project_config_id, external_task_ref, review_target_pr_url)
    WHERE work_type = 'pull_request_review'
      AND review_target_pr_url IS NOT NULL
      AND status IN ('pending', 'dispatched', 'running');
