CREATE TYPE integration_type AS ENUM ('kaneo');

ALTER TABLE project_configs ADD COLUMN integration_type integration_type NOT NULL DEFAULT 'kaneo';
