ALTER TABLE project_configs
    DROP COLUMN primary_model_provider_key,
    DROP COLUMN primary_model_id,
    DROP COLUMN small_model_provider_key,
    DROP COLUMN small_model_id;
