ALTER TABLE project_configs
    ADD COLUMN primary_model_provider_key TEXT,
    ADD COLUMN primary_model_id TEXT,
    ADD COLUMN small_model_provider_key TEXT,
    ADD COLUMN small_model_id TEXT;
