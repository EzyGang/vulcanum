CREATE INDEX idx_task_prs_repo_number
    ON task_prs (LOWER(repo_full_name), pr_number);
