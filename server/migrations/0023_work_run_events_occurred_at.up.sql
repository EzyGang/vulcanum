ALTER TABLE work_run_events ADD COLUMN occurred_at TIMESTAMPTZ;

UPDATE work_run_events SET occurred_at = created_at WHERE occurred_at IS NULL;

ALTER TABLE work_run_events ALTER COLUMN occurred_at SET NOT NULL;

CREATE INDEX idx_work_run_events_occurred_at ON work_run_events(work_run_id, occurred_at);