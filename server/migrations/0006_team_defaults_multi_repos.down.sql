UPDATE project_configs SET prompt_template = '' WHERE prompt_template IS NULL;
UPDATE project_configs SET agents_md = '' WHERE agents_md IS NULL;

ALTER TABLE project_configs
    ADD COLUMN opencode_config TEXT NOT NULL DEFAULT '',
    ALTER COLUMN prompt_template SET NOT NULL,
    ALTER COLUMN agents_md SET NOT NULL;

DROP TABLE work_run_prs;
DROP TABLE work_run_repos;
DROP TABLE project_config_repos;

ALTER TABLE teams
    DROP COLUMN small_model_id,
    DROP COLUMN small_model_provider_key,
    DROP COLUMN primary_model_id,
    DROP COLUMN primary_model_provider_key,
    DROP COLUMN agents_md,
    DROP COLUMN prompt_template;
