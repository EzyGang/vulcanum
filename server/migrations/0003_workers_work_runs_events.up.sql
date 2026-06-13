CREATE TYPE worker_status AS ENUM ('idle', 'busy', 'disconnected', 'unhealthy');

CREATE TYPE work_run_status AS ENUM (
    'pending',
    'dispatched',
    'running',
    'completed',
    'failed',
    'stalled'
);

CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    last_seen TIMESTAMPTZ,
    status worker_status NOT NULL DEFAULT 'idle',
    capabilities JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    refresh_expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    active_jobs INTEGER NOT NULL DEFAULT 0,
    max_concurrent_jobs INTEGER NOT NULL DEFAULT 3,
    consecutive_errors INT NOT NULL DEFAULT 0,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE
);

CREATE TABLE work_runs (
    id UUID PRIMARY KEY,
    external_task_ref TEXT NOT NULL,
    project_config_id UUID NOT NULL REFERENCES project_configs(id),
    worker_id UUID REFERENCES workers(id) ON DELETE SET NULL,
    status work_run_status NOT NULL DEFAULT 'pending',
    prompt_text TEXT NOT NULL,
    result_pr_url TEXT,
    result_exit_code INTEGER,
    tokens_used BIGINT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    repo_url TEXT NOT NULL DEFAULT '',
    agents_md TEXT NOT NULL DEFAULT '',
    input_tokens BIGINT DEFAULT 0,
    output_tokens BIGINT DEFAULT 0,
    cache_read_tokens BIGINT DEFAULT 0,
    cache_write_tokens BIGINT DEFAULT 0,
    model_used TEXT,
    finish_status TEXT,
    finish_summary TEXT,
    finish_blocked_reason TEXT,
    finish_next_column TEXT,
    task_title TEXT,
    task_slug TEXT,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_active_task
    ON work_runs(project_config_id, external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running');

CREATE TABLE work_run_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    sequence BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    occurred_at TIMESTAMPTZ NOT NULL,
    UNIQUE (work_run_id, sequence)
);

CREATE INDEX idx_work_run_events_run_sequence
    ON work_run_events(work_run_id, sequence);

CREATE INDEX idx_work_run_events_occurred_at
    ON work_run_events(work_run_id, occurred_at);

CREATE TRIGGER trg_work_runs_updated_at
    BEFORE UPDATE ON work_runs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
