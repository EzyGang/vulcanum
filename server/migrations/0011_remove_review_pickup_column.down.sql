ALTER TABLE teams
    ADD COLUMN review_pickup_column TEXT NOT NULL DEFAULT 'in-review';

ALTER TABLE project_configs
    ADD COLUMN review_pickup_column TEXT;
