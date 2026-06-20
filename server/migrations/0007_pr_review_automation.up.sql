CREATE TYPE work_run_type AS ENUM ('implementation', 'pull_request_review');

ALTER TABLE teams
    ADD COLUMN review_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN review_pickup_column TEXT NOT NULL DEFAULT 'in-review',
    ADD COLUMN review_max_turns INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN review_prompt_template TEXT NOT NULL DEFAULT 'Review this pull request for the linked task.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}

Follow the repository AGENTS.md instructions. Review code quality, correctness, maintainability, and project conventions. Do not edit files, commit, push, or create pull requests. Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. Include this marker in the review body: {{review_marker}}. If the marker already exists on the pull request, do not post another review.

The review body must use exactly these Markdown sections in this order:
## CRITICAL
- List defects that make the implementation unsafe, incorrect, or unusable. Use "- None" if empty.

## WARNINGS
- List defects that should be fixed before merging. Use "- None" if empty.

## SUGGESTIONS
- List optional improvements. Use "- None" if empty.

When done, call finish_run with status completed, review_url if available, review_body, and review_already_exists.';

ALTER TABLE teams
    ALTER COLUMN review_prompt_template DROP DEFAULT;

UPDATE teams
SET review_prompt_template = replace(review_prompt_template, E'\r\n', E'\n')
WHERE review_prompt_template LIKE 'Review this pull request for the linked task.%';

ALTER TABLE project_configs
    ADD COLUMN review_enabled BOOLEAN,
    ADD COLUMN review_pickup_column TEXT,
    ADD COLUMN review_max_turns INTEGER,
    ADD COLUMN review_prompt_template TEXT;

ALTER TABLE work_runs
    ADD COLUMN work_type work_run_type NOT NULL DEFAULT 'implementation',
    ADD COLUMN parent_work_run_id UUID REFERENCES work_runs(id) ON DELETE SET NULL,
    ADD COLUMN task_body TEXT NOT NULL DEFAULT '',
    ADD COLUMN review_target_pr_url TEXT,
    ADD COLUMN review_target_repo_full_name TEXT,
    ADD COLUMN review_url TEXT,
    ADD COLUMN review_body TEXT,
    ADD COLUMN review_already_exists BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE task_prs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    external_task_ref TEXT NOT NULL,
    pr_url TEXT NOT NULL,
    repo_full_name TEXT NOT NULL,
    pr_number BIGINT NOT NULL,
    source_work_run_id UUID REFERENCES work_runs(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_config_id, external_task_ref, pr_url)
);

CREATE TABLE work_run_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    pr_url TEXT NOT NULL,
    repo_full_name TEXT NOT NULL,
    review_url TEXT,
    review_body TEXT,
    review_already_exists BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_run_id, pr_url)
);

DROP INDEX unique_active_task;

CREATE UNIQUE INDEX unique_active_implementation_task
    ON work_runs(project_config_id, external_task_ref)
    WHERE status IN ('pending', 'dispatched', 'running') AND work_type = 'implementation';

CREATE UNIQUE INDEX unique_active_review_run_per_task_pr
    ON work_runs(project_config_id, external_task_ref, review_target_pr_url)
    WHERE work_type = 'pull_request_review'
      AND review_target_pr_url IS NOT NULL
      AND status IN ('pending', 'dispatched', 'running');

CREATE TRIGGER trg_task_prs_updated_at
    BEFORE UPDATE ON task_prs FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
