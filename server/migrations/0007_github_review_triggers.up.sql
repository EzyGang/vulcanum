ALTER TABLE work_runs
    ADD COLUMN github_installation_id BIGINT,
    ADD COLUMN github_delivery_id TEXT;

CREATE UNIQUE INDEX unique_work_run_github_delivery
    ON work_runs (github_delivery_id)
    WHERE github_delivery_id IS NOT NULL;

CREATE TABLE github_review_tickets (
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    pr_number BIGINT NOT NULL,
    external_task_ref TEXT,
    creation_token UUID NOT NULL,
    creation_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (project_config_id, repo_full_name, pr_number)
);
