DROP INDEX IF EXISTS idx_work_run_events_occurred_at;

ALTER TABLE work_run_events DROP COLUMN occurred_at;