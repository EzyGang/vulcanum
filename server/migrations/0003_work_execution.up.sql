CREATE TYPE worker_status AS ENUM ('idle', 'busy', 'disconnected', 'unhealthy');
CREATE TYPE work_run_status AS ENUM (
    'pending',
    'dispatched',
    'running',
    'completed',
    'failed',
    'stalled'
);
CREATE TYPE work_run_type AS ENUM ('implementation', 'pull_request_review');

CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    last_seen TIMESTAMPTZ,
    status worker_status NOT NULL DEFAULT 'idle',
    capabilities JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    refresh_expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 days',
    active_jobs INTEGER NOT NULL DEFAULT 0,
    max_concurrent_jobs INTEGER NOT NULL DEFAULT 3,
    consecutive_errors INTEGER NOT NULL DEFAULT 0,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE
);

CREATE TABLE work_runs (
    id UUID PRIMARY KEY,
    external_task_ref TEXT NOT NULL,
    project_config_id UUID NOT NULL REFERENCES project_configs(id),
    worker_id UUID REFERENCES workers(id) ON DELETE SET NULL,
    status work_run_status NOT NULL DEFAULT 'pending',
    result_pr_url TEXT,
    result_exit_code INTEGER,
    tokens_used BIGINT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    input_tokens BIGINT DEFAULT 0,
    output_tokens BIGINT DEFAULT 0,
    cache_read_tokens BIGINT DEFAULT 0,
    cache_write_tokens BIGINT DEFAULT 0,
    model_used TEXT,
    finish_status TEXT,
    result_summary TEXT,
    finish_blocked_reason TEXT,
    finish_next_column TEXT,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    work_type work_run_type NOT NULL DEFAULT 'implementation',
    parent_work_run_id UUID REFERENCES work_runs(id) ON DELETE SET NULL,
    review_target_pr_url TEXT,
    review_target_repo_full_name TEXT,
    task_title TEXT,
    task_slug TEXT
);

CREATE INDEX idx_work_runs_team_config_task_created
    ON work_runs (team_id, project_config_id, external_task_ref, created_at DESC, id DESC);
CREATE UNIQUE INDEX unique_active_implementation_task
    ON work_runs (project_config_id, external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running')
      AND work_type = 'implementation';
CREATE UNIQUE INDEX unique_active_review_run_per_task_pr
    ON work_runs (project_config_id, external_task_ref, review_target_pr_url)
    WHERE work_type = 'pull_request_review'
      AND review_target_pr_url IS NOT NULL
      AND status IN ('pending', 'dispatched', 'running');

CREATE TRIGGER trg_work_runs_updated_at
BEFORE UPDATE ON work_runs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE work_run_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    sequence BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    occurred_at TIMESTAMPTZ NOT NULL,
    UNIQUE (work_run_id, sequence)
);

CREATE INDEX idx_work_run_events_occurred_at
    ON work_run_events (work_run_id, occurred_at);
CREATE INDEX idx_work_run_events_run_sequence
    ON work_run_events (work_run_id, sequence);

CREATE TABLE work_run_prs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    pr_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, position),
    UNIQUE (work_run_id, pr_url)
);

CREATE TABLE work_run_repos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, position),
    UNIQUE (work_run_id, repo_full_name)
);

CREATE TABLE work_run_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    pr_url TEXT NOT NULL,
    repo_full_name TEXT NOT NULL,
    review_url TEXT,
    review_body TEXT,
    review_already_exists BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, pr_url)
);

CREATE TABLE task_prs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    external_task_ref TEXT NOT NULL,
    pr_url TEXT NOT NULL,
    repo_full_name TEXT NOT NULL,
    pr_number BIGINT NOT NULL,
    source_work_run_id UUID REFERENCES work_runs(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_config_id, external_task_ref, pr_url)
);

CREATE INDEX idx_task_prs_repo_number ON task_prs (LOWER(repo_full_name), pr_number);

CREATE TRIGGER trg_task_prs_updated_at
BEFORE UPDATE ON task_prs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE task_augmentations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    external_task_ref TEXT NOT NULL,
    tokens_used BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_write_tokens BIGINT NOT NULL DEFAULT 0,
    finished_runs_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, project_config_id, external_task_ref)
);

CREATE INDEX idx_task_augmentations_project_task
    ON task_augmentations (team_id, project_config_id, external_task_ref);

CREATE TRIGGER trg_task_augmentations_updated_at
BEFORE UPDATE ON task_augmentations
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
