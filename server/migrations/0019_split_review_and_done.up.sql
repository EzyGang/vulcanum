ALTER TABLE project_configs
    RENAME COLUMN target_column TO review_column;

ALTER TABLE project_configs
    ADD COLUMN done_column TEXT;

UPDATE project_configs
SET done_column = review_column;

ALTER TABLE project_configs
    ALTER COLUMN done_column SET NOT NULL,
    ALTER COLUMN done_column SET DEFAULT 'done';
