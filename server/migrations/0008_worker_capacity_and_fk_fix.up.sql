ALTER TABLE work_runs
    DROP CONSTRAINT IF EXISTS work_runs_worker_id_fkey;

ALTER TABLE work_runs
    ADD CONSTRAINT work_runs_worker_id_fkey
    FOREIGN KEY (worker_id) REFERENCES workers(id)
    ON DELETE SET NULL;

ALTER TABLE workers
    ADD COLUMN active_jobs INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN max_concurrent_jobs INTEGER NOT NULL DEFAULT 3;
