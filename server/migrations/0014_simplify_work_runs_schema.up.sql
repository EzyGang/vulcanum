ALTER TABLE work_runs
    DROP COLUMN IF EXISTS task_body,
    DROP COLUMN IF EXISTS task_title,
    DROP COLUMN IF EXISTS task_slug,
    DROP COLUMN IF EXISTS prompt_text,
    DROP COLUMN IF EXISTS agents_md,
    DROP COLUMN IF EXISTS repo_url,
    DROP COLUMN IF EXISTS review_url,
    DROP COLUMN IF EXISTS review_body,
    DROP COLUMN IF EXISTS review_already_exists;

ALTER TABLE work_runs
    RENAME COLUMN finish_summary TO result_summary;
