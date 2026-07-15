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
    ON integration_providers (team_id, name);

CREATE TABLE project_configs (
    id UUID PRIMARY KEY,
    external_project_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    pickup_column TEXT NOT NULL DEFAULT 'todo',
    review_column TEXT NOT NULL DEFAULT 'in review',
    progress_column TEXT NOT NULL DEFAULT 'in progress',
    prompt_template TEXT,
    repo_url TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    agents_md TEXT DEFAULT '',
    external_workspace_id TEXT NOT NULL DEFAULT '',
    integration_type integration_type NOT NULL DEFAULT 'kaneo',
    provider_id UUID REFERENCES integration_providers(id) ON DELETE SET NULL,
    max_turns INTEGER NOT NULL DEFAULT 3,
    name TEXT NOT NULL DEFAULT '',
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    review_enabled BOOLEAN,
    review_max_turns INTEGER,
    review_prompt_template TEXT,
    max_in_progress_tasks INTEGER,
    done_column TEXT NOT NULL DEFAULT 'done'
);

CREATE UNIQUE INDEX project_configs_team_provider_external_key
    ON project_configs (team_id, provider_id, external_project_id);

CREATE TRIGGER trg_project_configs_updated_at
BEFORE UPDATE ON project_configs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE project_config_repos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_config_id, position),
    UNIQUE (project_config_id, repo_full_name)
);
