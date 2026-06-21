ALTER TABLE project_configs
    DROP COLUMN max_in_progress_tasks;

ALTER TABLE teams
    DROP COLUMN max_in_progress_tasks;
