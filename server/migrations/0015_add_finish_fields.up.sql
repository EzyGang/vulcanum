ALTER TABLE work_runs
    ADD COLUMN finish_status TEXT,
    ADD COLUMN finish_summary TEXT,
    ADD COLUMN finish_blocked_reason TEXT,
    ADD COLUMN finish_next_column TEXT;
