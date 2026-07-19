ALTER TABLE project_usage_daily
    ADD COLUMN implementation_runs_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN review_runs_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN successful_runs_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN failed_runs_count BIGINT NOT NULL DEFAULT 0;

INSERT INTO project_usage_daily (
    project_config_id,
    usage_date,
    tokens_used,
    input_tokens,
    output_tokens,
    cache_read_tokens,
    cache_write_tokens,
    finished_runs_count,
    implementation_runs_count,
    review_runs_count,
    successful_runs_count,
    failed_runs_count
)
SELECT
    project_config_id,
    (updated_at AT TIME ZONE 'UTC')::DATE,
    0,
    0,
    0,
    0,
    0,
    COUNT(*),
    COUNT(*) FILTER (WHERE work_type = 'implementation'),
    COUNT(*) FILTER (WHERE work_type = 'pull_request_review'),
    COUNT(*) FILTER (WHERE status = 'completed'),
    COUNT(*) FILTER (WHERE status = 'failed')
FROM work_runs
WHERE status IN ('completed', 'failed')
GROUP BY project_config_id, (updated_at AT TIME ZONE 'UTC')::DATE
ON CONFLICT (project_config_id, usage_date) DO UPDATE SET
    finished_runs_count = EXCLUDED.finished_runs_count,
    implementation_runs_count = EXCLUDED.implementation_runs_count,
    review_runs_count = EXCLUDED.review_runs_count,
    successful_runs_count = EXCLUDED.successful_runs_count,
    failed_runs_count = EXCLUDED.failed_runs_count;
