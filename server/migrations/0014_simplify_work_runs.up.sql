ALTER TABLE work_runs
    DROP COLUMN task_body,
    DROP COLUMN task_title,
    DROP COLUMN task_slug,
    DROP COLUMN prompt_text,
    DROP COLUMN repo_url,
    DROP COLUMN agents_md,
    DROP COLUMN review_url,
    DROP COLUMN review_body,
    DROP COLUMN review_already_exists;

ALTER TABLE work_runs
    RENAME COLUMN finish_summary TO result_summary;
