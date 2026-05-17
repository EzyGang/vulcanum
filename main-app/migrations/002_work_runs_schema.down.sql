DROP TRIGGER IF EXISTS trg_work_runs_updated_at ON work_runs;
DROP TRIGGER IF EXISTS trg_project_configs_updated_at ON project_configs;
DROP FUNCTION IF EXISTS update_updated_at_column CASCADE;
DROP TABLE IF EXISTS work_runs CASCADE;
DROP TABLE IF EXISTS workers CASCADE;
DROP TABLE IF EXISTS project_configs CASCADE;
DROP TYPE IF EXISTS work_run_status;
DROP TYPE IF EXISTS worker_status;
