CREATE TYPE integration_type AS ENUM ('kaneo');

CREATE TABLE integration_providers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type integration_type NOT NULL DEFAULT 'kaneo',
    instance_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX integration_providers_team_name_key
    ON integration_providers(team_id, name);

CREATE TABLE project_configs (
    id UUID PRIMARY KEY,
    external_project_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    pickup_column TEXT NOT NULL DEFAULT 'todo',
    target_column TEXT NOT NULL DEFAULT 'in review',
    progress_column TEXT NOT NULL DEFAULT 'in progress',
    prompt_template TEXT NOT NULL,
    repo_url TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    agents_md TEXT NOT NULL DEFAULT '',
    external_workspace_id TEXT NOT NULL DEFAULT '',
    integration_type integration_type NOT NULL DEFAULT 'kaneo',
    provider_id UUID REFERENCES integration_providers(id) ON DELETE SET NULL,
    opencode_config TEXT NOT NULL DEFAULT '',
    blocked_column TEXT NOT NULL DEFAULT 'Blocked',
    max_turns INTEGER NOT NULL DEFAULT 3,
    name TEXT NOT NULL DEFAULT '',
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX project_configs_team_provider_external_key
    ON project_configs(team_id, provider_id, external_project_id);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trg_project_configs_updated_at
    BEFORE UPDATE ON project_configs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE github_installations (
    id BIGSERIAL PRIMARY KEY,
    github_installation_id BIGINT NOT NULL UNIQUE,
    account_login TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    installed_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_github_installations_team_id
    ON github_installations(team_id);
