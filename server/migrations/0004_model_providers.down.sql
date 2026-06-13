ALTER TABLE project_configs
    DROP COLUMN small_model_id,
    DROP COLUMN small_model_provider_key,
    DROP COLUMN primary_model_id,
    DROP COLUMN primary_model_provider_key;

DROP TRIGGER IF EXISTS trg_model_provider_configs_updated_at ON model_provider_configs;
DROP TABLE model_provider_configs;
