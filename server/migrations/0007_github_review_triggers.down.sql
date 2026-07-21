DROP INDEX IF EXISTS unique_work_run_github_delivery;

ALTER TABLE work_runs
    DROP COLUMN IF EXISTS github_delivery_id,
    DROP COLUMN IF EXISTS github_installation_id;
