CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    personal_user_id TEXT UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    prompt_template TEXT NOT NULL,
    agents_md TEXT NOT NULL DEFAULT '',
    primary_model_provider_key TEXT,
    primary_model_id TEXT,
    small_model_provider_key TEXT,
    small_model_id TEXT,
    review_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    review_max_turns INTEGER NOT NULL DEFAULT 1,
    review_prompt_template TEXT NOT NULL,
    max_in_progress_tasks INTEGER NOT NULL DEFAULT 1,
    agent_backend TEXT NOT NULL DEFAULT 'opencode'
);

INSERT INTO teams (id, name, prompt_template, review_prompt_template)
VALUES ('00000000-0000-0000-0000-000000000001', 'Default team', '', '');

CREATE TABLE team_members (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE user_identities (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    provider_login TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    provider_verified_at TIMESTAMPTZ,
    UNIQUE (provider, provider_user_id)
);

CREATE TABLE user_refresh_tokens (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_user_refresh_tokens_active_hash
    ON user_refresh_tokens (token_hash)
    WHERE revoked_at IS NULL;
CREATE INDEX idx_user_refresh_tokens_user_id ON user_refresh_tokens (user_id);

CREATE TABLE github_installations (
    id BIGSERIAL PRIMARY KEY,
    github_installation_id BIGINT NOT NULL UNIQUE,
    account_login TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    installed_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_github_installations_team_id ON github_installations (team_id);

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
    ON model_provider_configs (team_id, provider_key);

CREATE TRIGGER trg_model_provider_configs_updated_at
BEFORE UPDATE ON model_provider_configs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
