ALTER TABLE teams
    ADD COLUMN max_in_progress_tasks INTEGER NOT NULL DEFAULT 1;

ALTER TABLE project_configs
    ADD COLUMN max_in_progress_tasks INTEGER;
