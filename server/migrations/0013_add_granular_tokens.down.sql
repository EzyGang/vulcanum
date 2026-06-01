ALTER TABLE work_runs DROP COLUMN IF EXISTS input_tokens;
ALTER TABLE work_runs DROP COLUMN IF EXISTS output_tokens;
ALTER TABLE work_runs DROP COLUMN IF EXISTS cache_read_tokens;
ALTER TABLE work_runs DROP COLUMN IF EXISTS cache_write_tokens;
ALTER TABLE work_runs DROP COLUMN IF EXISTS model_used;