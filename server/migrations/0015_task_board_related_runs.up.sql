CREATE INDEX idx_work_runs_team_config_task_created
    ON work_runs(team_id, project_config_id, external_task_ref, created_at DESC, id DESC);
