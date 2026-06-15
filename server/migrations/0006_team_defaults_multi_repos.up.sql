ALTER TABLE teams
    ADD COLUMN prompt_template TEXT NOT NULL DEFAULT 'Review {{task_title}}

{{task_body}}

Repositories:
{{repo_urls}}

Follow the repository instructions and keep the final response concise.',
    ADD COLUMN agents_md TEXT NOT NULL DEFAULT '',
    ADD COLUMN primary_model_provider_key TEXT,
    ADD COLUMN primary_model_id TEXT,
    ADD COLUMN small_model_provider_key TEXT,
    ADD COLUMN small_model_id TEXT;

CREATE TABLE project_config_repos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_config_id, repo_full_name),
    UNIQUE (project_config_id, position)
);

INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position)
SELECT
    id,
    regexp_replace(regexp_replace(repo_url, '^https?://github.com/', ''), '\.git$', ''),
    repo_url,
    0
FROM project_configs
WHERE repo_url <> '';

CREATE TABLE work_run_repos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    repo_full_name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, repo_full_name),
    UNIQUE (work_run_id, position)
);

INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position)
SELECT
    id,
    regexp_replace(regexp_replace(repo_url, '^https?://github.com/', ''), '\.git$', ''),
    repo_url,
    0
FROM work_runs
WHERE repo_url <> '';

CREATE TABLE work_run_prs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    pr_url TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, pr_url),
    UNIQUE (work_run_id, position)
);

ALTER TABLE project_configs
    ALTER COLUMN prompt_template DROP NOT NULL,
    ALTER COLUMN agents_md DROP NOT NULL,
    DROP COLUMN opencode_config;
