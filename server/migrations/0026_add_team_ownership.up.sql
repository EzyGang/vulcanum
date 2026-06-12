CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    personal_user_id TEXT UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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
    UNIQUE (provider, provider_user_id)
);

INSERT INTO teams (id, name)
VALUES ('00000000-0000-0000-0000-000000000001', 'Default team');

INSERT INTO team_members (team_id, user_id, role)
SELECT '00000000-0000-0000-0000-000000000001', id, 'owner'
FROM users
ON CONFLICT DO NOTHING;

ALTER TABLE integration_providers
    ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE;

UPDATE integration_providers
SET team_id = '00000000-0000-0000-0000-000000000001'
WHERE team_id IS NULL;

ALTER TABLE integration_providers
    ALTER COLUMN team_id SET NOT NULL;

ALTER TABLE project_configs
    ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE;

UPDATE project_configs
SET team_id = '00000000-0000-0000-0000-000000000001'
WHERE team_id IS NULL;

ALTER TABLE project_configs
    ALTER COLUMN team_id SET NOT NULL;

ALTER TABLE github_installations
    ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    ADD COLUMN installed_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;

UPDATE github_installations
SET team_id = '00000000-0000-0000-0000-000000000001'
WHERE team_id IS NULL;

ALTER TABLE github_installations
    ALTER COLUMN team_id SET NOT NULL;

ALTER TABLE workers
    ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE;

UPDATE workers
SET team_id = '00000000-0000-0000-0000-000000000001'
WHERE team_id IS NULL;

ALTER TABLE workers
    ALTER COLUMN team_id SET NOT NULL;

ALTER TABLE work_runs
    ADD COLUMN team_id UUID REFERENCES teams(id) ON DELETE CASCADE;

UPDATE work_runs wr
SET team_id = pc.team_id
FROM project_configs pc
WHERE wr.project_config_id = pc.id
  AND wr.team_id IS NULL;

UPDATE work_runs
SET team_id = '00000000-0000-0000-0000-000000000001'
WHERE team_id IS NULL;

ALTER TABLE work_runs
    ALTER COLUMN team_id SET NOT NULL;

ALTER TABLE integration_providers
    DROP CONSTRAINT IF EXISTS integration_providers_name_key;

CREATE UNIQUE INDEX integration_providers_team_name_key
    ON integration_providers(team_id, name);

ALTER TABLE project_configs
    DROP CONSTRAINT IF EXISTS project_configs_provider_external_key;

ALTER TABLE project_configs
    DROP CONSTRAINT IF EXISTS project_configs_kaneo_project_id_key;

CREATE UNIQUE INDEX project_configs_team_provider_external_key
    ON project_configs(team_id, provider_id, external_project_id);

DROP INDEX IF EXISTS unique_active_task;

CREATE UNIQUE INDEX unique_active_task
    ON work_runs (project_config_id, external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running');

DROP INDEX IF EXISTS idx_github_installations_account_login;

CREATE INDEX idx_github_installations_team_id
    ON github_installations(team_id);
