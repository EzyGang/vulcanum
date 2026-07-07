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
    ON task_augmentations(team_id, project_config_id, external_task_ref);

CREATE TRIGGER trg_task_augmentations_updated_at
    BEFORE UPDATE ON task_augmentations FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
