DROP INDEX IF EXISTS unique_active_review_run_per_task_pr;

CREATE UNIQUE INDEX IF NOT EXISTS unique_review_run_per_task_pr
    ON work_runs(project_config_id, external_task_ref, review_target_pr_url)
    WHERE work_type = 'pull_request_review'
      AND review_target_pr_url IS NOT NULL;

UPDATE teams
SET review_prompt_template = 'Review this pull request for the linked task.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}

Follow the repository AGENTS.md instructions. Review code quality, correctness, maintainability, and project conventions. Do not edit files, commit, push, or create pull requests. Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. Include this marker in the review body: {{review_marker}}. If the marker already exists on the pull request, do not post another review. When done, call finish_run with status completed, review_url if available, review_body, and review_already_exists.'
WHERE review_prompt_template = 'Review this pull request for the linked task.

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
    ALTER COLUMN review_prompt_template SET DEFAULT 'Review this pull request for the linked task.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}

Follow the repository AGENTS.md instructions. Review code quality, correctness, maintainability, and project conventions. Do not edit files, commit, push, or create pull requests. Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. Include this marker in the review body: {{review_marker}}. If the marker already exists on the pull request, do not post another review. When done, call finish_run with status completed, review_url if available, review_body, and review_already_exists.';
