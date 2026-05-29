CREATE TABLE integration_providers (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  provider_type integration_type NOT NULL DEFAULT 'kaneo',
  instance_url TEXT NOT NULL,
  api_key TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE project_configs ADD COLUMN provider_id UUID
  REFERENCES integration_providers(id) ON DELETE SET NULL;
