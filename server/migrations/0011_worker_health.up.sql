ALTER TYPE worker_status ADD VALUE 'unhealthy';

ALTER TABLE workers ADD COLUMN consecutive_errors INT NOT NULL DEFAULT 0;