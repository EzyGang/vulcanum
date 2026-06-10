DROP INDEX IF EXISTS idx_github_installations_team_id;

CREATE UNIQUE INDEX idx_github_installations_account_login
    ON github_installations(account_login);

DROP INDEX IF EXISTS unique_active_task;

CREATE UNIQUE INDEX unique_active_task
    ON work_runs (external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running');

DROP INDEX IF EXISTS project_configs_team_provider_external_key;

ALTER TABLE project_configs
    ADD CONSTRAINT project_configs_provider_external_key UNIQUE (provider_id, external_project_id);

DROP INDEX IF EXISTS integration_providers_team_name_key;

ALTER TABLE integration_providers
    ADD CONSTRAINT integration_providers_name_key UNIQUE (name);

ALTER TABLE work_runs DROP COLUMN IF EXISTS team_id;
ALTER TABLE workers DROP COLUMN IF EXISTS team_id;
ALTER TABLE github_installations DROP COLUMN IF EXISTS installed_by_user_id;
ALTER TABLE github_installations DROP COLUMN IF EXISTS team_id;
ALTER TABLE project_configs DROP COLUMN IF EXISTS team_id;
ALTER TABLE integration_providers DROP COLUMN IF EXISTS team_id;

DROP TABLE IF EXISTS user_identities;
DROP TABLE IF EXISTS team_members;
DROP TABLE IF EXISTS teams;
