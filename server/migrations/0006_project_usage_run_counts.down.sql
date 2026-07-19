ALTER TABLE project_usage_daily
    DROP COLUMN failed_runs_count,
    DROP COLUMN successful_runs_count,
    DROP COLUMN review_runs_count,
    DROP COLUMN implementation_runs_count;
