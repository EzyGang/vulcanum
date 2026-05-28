ALTER TABLE project_configs DROP COLUMN IF EXISTS integration_type;

DROP TYPE IF EXISTS integration_type;
