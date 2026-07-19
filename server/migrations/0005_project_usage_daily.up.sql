CREATE TABLE project_usage_daily (
    project_config_id UUID NOT NULL REFERENCES project_configs(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    tokens_used BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_write_tokens BIGINT NOT NULL DEFAULT 0,
    finished_runs_count BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (project_config_id, usage_date)
);

INSERT INTO project_usage_daily (
    project_config_id,
    usage_date,
    tokens_used,
    input_tokens,
    output_tokens,
    cache_read_tokens,
    cache_write_tokens,
    finished_runs_count
)
SELECT
    project_config_id,
    (statement_timestamp() AT TIME ZONE 'UTC')::DATE,
    SUM(tokens_used),
    SUM(input_tokens),
    SUM(output_tokens),
    SUM(cache_read_tokens),
    SUM(cache_write_tokens),
    SUM(finished_runs_count)
FROM task_augmentations
GROUP BY project_config_id;
