UPDATE workers SET status = 'idle'::worker_status WHERE status = 'unhealthy'::worker_status;

ALTER TABLE workers DROP COLUMN consecutive_errors;