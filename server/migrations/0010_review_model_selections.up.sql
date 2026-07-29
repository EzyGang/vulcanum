ALTER TABLE teams
    ADD COLUMN review_primary_model_provider_key TEXT,
    ADD COLUMN review_primary_model_id TEXT,
    ADD COLUMN review_small_model_provider_key TEXT,
    ADD COLUMN review_small_model_id TEXT;
