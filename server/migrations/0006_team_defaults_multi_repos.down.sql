UPDATE project_configs pc
SET prompt_template = t.prompt_template
FROM teams t
WHERE pc.team_id = t.id AND pc.prompt_template IS NULL;

UPDATE project_configs pc
SET agents_md = t.agents_md
FROM teams t
WHERE pc.team_id = t.id AND pc.agents_md IS NULL;

ALTER TABLE project_configs
    ADD COLUMN opencode_config TEXT NOT NULL DEFAULT '',
    ALTER COLUMN prompt_template SET NOT NULL,
    ALTER COLUMN agents_md SET NOT NULL;

DROP TABLE IF EXISTS work_run_prs;
DROP TABLE IF EXISTS work_run_repos;
DROP TABLE IF EXISTS project_config_repos;

ALTER TABLE teams
    DROP COLUMN IF EXISTS small_model_id,
    DROP COLUMN IF EXISTS small_model_provider_key,
    DROP COLUMN IF EXISTS primary_model_id,
    DROP COLUMN IF EXISTS primary_model_provider_key,
    DROP COLUMN IF EXISTS agents_md,
    DROP COLUMN IF EXISTS prompt_template;
