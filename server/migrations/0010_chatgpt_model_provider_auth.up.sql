ALTER TABLE model_provider_configs
    ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'api_key',
    ADD COLUMN oauth_credentials JSONB,
    ADD COLUMN oauth_metadata JSONB NOT NULL DEFAULT '{}';

DROP INDEX IF EXISTS model_provider_configs_team_provider_key;

CREATE UNIQUE INDEX model_provider_configs_team_provider_auth_type
    ON model_provider_configs(team_id, provider_key, auth_type);

ALTER TABLE project_configs
    ADD COLUMN primary_model_provider_config_id UUID REFERENCES model_provider_configs(id) ON DELETE SET NULL,
    ADD COLUMN small_model_provider_config_id UUID REFERENCES model_provider_configs(id) ON DELETE SET NULL;

UPDATE project_configs pc
SET primary_model_provider_config_id = mp.id
FROM model_provider_configs mp
WHERE mp.team_id = pc.team_id
    AND mp.provider_key = pc.primary_model_provider_key
    AND mp.auth_type = 'api_key';

UPDATE project_configs pc
SET small_model_provider_config_id = mp.id
FROM model_provider_configs mp
WHERE mp.team_id = pc.team_id
    AND mp.provider_key = pc.small_model_provider_key
    AND mp.auth_type = 'api_key';

ALTER TABLE teams
    ADD COLUMN primary_model_provider_config_id UUID REFERENCES model_provider_configs(id) ON DELETE SET NULL,
    ADD COLUMN small_model_provider_config_id UUID REFERENCES model_provider_configs(id) ON DELETE SET NULL;

UPDATE teams t
SET primary_model_provider_config_id = mp.id
FROM model_provider_configs mp
WHERE mp.team_id = t.id
    AND mp.provider_key = t.primary_model_provider_key
    AND mp.auth_type = 'api_key';

UPDATE teams t
SET small_model_provider_config_id = mp.id
FROM model_provider_configs mp
WHERE mp.team_id = t.id
    AND mp.provider_key = t.small_model_provider_key
    AND mp.auth_type = 'api_key';

CREATE TABLE model_provider_auth_attempts (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    encrypted_device_code JSONB NOT NULL,
    user_code TEXT NOT NULL,
    verification_uri TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    interval_seconds INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trg_model_provider_auth_attempts_updated_at
    BEFORE UPDATE ON model_provider_auth_attempts FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
