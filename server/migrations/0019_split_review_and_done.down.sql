ALTER TABLE project_configs
    DROP COLUMN done_column;

ALTER TABLE project_configs
    RENAME COLUMN review_column TO target_column;
