ALTER TABLE work_runs
    DROP COLUMN IF EXISTS agents_md,
    DROP COLUMN IF EXISTS repo_url;

ALTER TABLE project_configs
    DROP COLUMN IF EXISTS agents_md;
