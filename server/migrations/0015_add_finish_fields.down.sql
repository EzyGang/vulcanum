ALTER TABLE work_runs
    DROP COLUMN finish_status,
    DROP COLUMN finish_summary,
    DROP COLUMN finish_blocked_reason,
    DROP COLUMN finish_next_column;
