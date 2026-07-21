ALTER TABLE work_runs
    ADD COLUMN github_installation_id BIGINT,
    ADD COLUMN github_delivery_id TEXT;

CREATE UNIQUE INDEX unique_work_run_github_delivery
    ON work_runs (github_delivery_id)
    WHERE github_delivery_id IS NOT NULL;
