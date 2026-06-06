ALTER TABLE project_configs DROP CONSTRAINT IF EXISTS project_configs_provider_external_key;
ALTER TABLE project_configs ADD CONSTRAINT project_configs_kaneo_project_id_key UNIQUE (external_project_id);

ALTER TABLE project_configs RENAME COLUMN external_project_id TO kaneo_project_id;
ALTER TABLE project_configs RENAME COLUMN external_workspace_id TO kaneo_workspace_id;
