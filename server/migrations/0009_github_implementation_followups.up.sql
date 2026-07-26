CREATE TABLE github_implementation_followup_tickets (
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    pr_number BIGINT NOT NULL,
    external_task_ref TEXT,
    created_by_delivery_id TEXT,
    operation_token UUID,
    operation_delivery_id TEXT,
    operation_started_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (project_config_id, repo_full_name, pr_number)
);

CREATE TRIGGER trg_github_implementation_followup_tickets_updated_at
BEFORE UPDATE ON github_implementation_followup_tickets
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE github_implementation_followup_requests (
    delivery_id TEXT PRIMARY KEY,
    github_installation_id BIGINT NOT NULL,
    repo_full_name TEXT NOT NULL,
    pr_number BIGINT NOT NULL,
    comment_id BIGINT NOT NULL,
    sender_id TEXT NOT NULL,
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    ticket_selector TEXT,
    external_task_ref TEXT,
    work_run_id UUID UNIQUE,
    request_body TEXT NOT NULL CHECK (LENGTH(BTRIM(request_body)) > 0),
    ticket_created BOOLEAN NOT NULL DEFAULT FALSE,
    ambiguous_task_refs TEXT[] NOT NULL DEFAULT '{}',
    outcome TEXT NOT NULL DEFAULT 'pending'
        CHECK (outcome IN ('pending', 'spawned', 'active_run', 'ambiguous_ticket', 'invalid_ticket')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_github_implementation_followup_requests_pr
    ON github_implementation_followup_requests
    (github_installation_id, LOWER(repo_full_name), pr_number, project_config_id);

CREATE TRIGGER trg_github_implementation_followup_requests_updated_at
BEFORE UPDATE ON github_implementation_followup_requests
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
