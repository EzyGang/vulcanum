ALTER TABLE workers
    DROP COLUMN IF EXISTS active_jobs,
    DROP COLUMN IF EXISTS max_concurrent_jobs;

ALTER TABLE work_runs
    DROP CONSTRAINT IF EXISTS work_runs_worker_id_fkey;

ALTER TABLE work_runs
    ADD CONSTRAINT work_runs_worker_id_fkey
    FOREIGN KEY (worker_id) REFERENCES workers(id);
