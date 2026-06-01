ALTER TABLE project_configs
    ADD COLUMN IF NOT EXISTS opencode_config TEXT NOT NULL DEFAULT '';