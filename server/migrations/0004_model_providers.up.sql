CREATE TABLE model_provider_configs (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    provider_key TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    credentials JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX model_provider_configs_team_provider_key
    ON model_provider_configs(team_id, provider_key);

CREATE TRIGGER trg_model_provider_configs_updated_at
    BEFORE UPDATE ON model_provider_configs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE project_configs
    ADD COLUMN primary_model_provider_key TEXT,
    ADD COLUMN primary_model_id TEXT,
    ADD COLUMN small_model_provider_key TEXT,
    ADD COLUMN small_model_id TEXT;
