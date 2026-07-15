UPDATE teams
SET prompt_template = $implementation$Implement the linked task.

Before editing, inspect the repository instructions and project manifests. Spawn a focused setup subagent when dependencies, generated files, or environment preparation are not already clear; have it follow the relevant AGENTS.md files and run the installation/setup commands the project needs.

Task title:
{{task_title}}

Task body:
```text
{{task_body}}
```

Repositories:
{{repo_urls}}

Follow the repository instructions, keep changes focused on the task, and keep the final response concise.$implementation$
WHERE prompt_template = '';

UPDATE teams
SET review_prompt_template = $review$Review this pull request for the linked task.

Before judging the implementation, inspect the repository instructions and project manifests. Follow every AGENTS.md file that applies to the changed directories. Spawn focused read-only subagents when they would help check correctness, tests, security, or project conventions; they must not edit files.

Review the solution for correctness, maintainability, and project fit. Make sure the implementation is elegant, avoids duplication, and has been formatted and validated with the repository commands that apply to the changed code. During the review phase, do not edit files, commit, push, or create pull requests. Keep the final response concise and focused on actionable findings.

Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. If a suitable review already exists for the current PR head commit, do not post a duplicate review. If the PR has new commits after the existing review, post a new review.

The review body must use exactly these Markdown sections in this order:
## CRITICAL
- List defects that make the implementation unsafe, incorrect, or unusable. Use "- None" if empty.

## WARNINGS
- List defects that should be fixed before merging, including missing or failing formatter, validation, or test commands. This includes serious violations of AGENTS.md guidelines. Use "- None" if empty.

## SUGGESTIONS
- List optional improvements. Use "- None" if empty.

When done, call finish_run with status completed, review_url if available, and review_body.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}$review$
WHERE review_prompt_template = '';
