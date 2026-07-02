-- Drop columns that can be reconstructed from source of truth
ALTER TABLE work_runs
    DROP COLUMN task_body,
    DROP COLUMN task_title,
    DROP COLUMN task_slug,
    DROP COLUMN prompt_text,
    DROP COLUMN agents_md,
    DROP COLUMN repo_url;

-- Drop review columns that live in work_run_reviews
ALTER TABLE work_runs
    DROP COLUMN review_url,
    DROP COLUMN review_body,
    DROP COLUMN review_already_exists;

-- Rename finish_summary to result_summary (unified field for implementation + review)
ALTER TABLE work_runs
    RENAME COLUMN finish_summary TO result_summary;
