CREATE TYPE worker_status AS ENUM ('idle', 'busy', 'disconnected');

CREATE TYPE work_run_status AS ENUM (
    'pending',
    'dispatched',
    'running',
    'completed',
    'failed',
    'stalled'
);

CREATE TABLE project_configs (
    id UUID PRIMARY KEY,
    kaneo_project_id TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    pickup_column TEXT NOT NULL DEFAULT 'todo',
    target_column TEXT NOT NULL DEFAULT 'in review',
    progress_column TEXT NOT NULL DEFAULT 'in progress',
    prompt_template TEXT NOT NULL,
    repo_url TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    access_token_hash TEXT NOT NULL,
    access_expires_at TIMESTAMPTZ NOT NULL,
    last_seen TIMESTAMPTZ,
    status worker_status NOT NULL DEFAULT 'idle',
    capabilities JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE work_runs (
    id UUID PRIMARY KEY,
    external_task_ref TEXT NOT NULL,
    project_config_id UUID NOT NULL REFERENCES project_configs(id),
    worker_id UUID REFERENCES workers(id),
    status work_run_status NOT NULL DEFAULT 'pending',
    prompt_text TEXT NOT NULL,
    result_pr_url TEXT,
    result_exit_code INTEGER,
    tokens_used INTEGER,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trg_work_runs_updated_at
    BEFORE UPDATE ON work_runs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_project_configs_updated_at
    BEFORE UPDATE ON project_configs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE UNIQUE INDEX unique_active_task ON work_runs (external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running');
