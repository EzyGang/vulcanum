ALTER TABLE project_configs RENAME COLUMN kaneo_project_id TO external_project_id;
ALTER TABLE project_configs RENAME COLUMN kaneo_workspace_id TO external_workspace_id;

ALTER TABLE project_configs DROP CONSTRAINT IF EXISTS project_configs_kaneo_project_id_key;
ALTER TABLE project_configs ADD CONSTRAINT project_configs_provider_external_key UNIQUE (provider_id, external_project_id);
