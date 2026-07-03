ALTER TABLE work_runs
    RENAME COLUMN result_summary TO finish_summary;

ALTER TABLE work_runs
    ADD COLUMN task_body TEXT NOT NULL DEFAULT '',
    ADD COLUMN task_title TEXT,
    ADD COLUMN task_slug TEXT,
    ADD COLUMN prompt_text TEXT NOT NULL DEFAULT '',
    ADD COLUMN agents_md TEXT NOT NULL DEFAULT '',
    ADD COLUMN repo_url TEXT NOT NULL DEFAULT '',
    ADD COLUMN review_body TEXT,
    ADD COLUMN review_url TEXT,
    ADD COLUMN review_already_exists BOOLEAN NOT NULL DEFAULT false;
