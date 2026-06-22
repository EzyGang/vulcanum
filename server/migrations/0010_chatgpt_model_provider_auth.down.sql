DROP TRIGGER IF EXISTS trg_model_provider_auth_attempts_updated_at ON model_provider_auth_attempts;
DROP TABLE IF EXISTS model_provider_auth_attempts;

ALTER TABLE teams
    DROP COLUMN IF EXISTS small_model_provider_config_id,
    DROP COLUMN IF EXISTS primary_model_provider_config_id;

ALTER TABLE project_configs
    DROP COLUMN IF EXISTS small_model_provider_config_id,
    DROP COLUMN IF EXISTS primary_model_provider_config_id;

DROP INDEX IF EXISTS model_provider_configs_team_provider_auth_type;

CREATE UNIQUE INDEX model_provider_configs_team_provider_key
    ON model_provider_configs(team_id, provider_key);

ALTER TABLE model_provider_configs
    DROP COLUMN IF EXISTS oauth_metadata,
    DROP COLUMN IF EXISTS oauth_credentials,
    DROP COLUMN IF EXISTS auth_type;
