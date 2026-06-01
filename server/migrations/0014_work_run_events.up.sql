CREATE TABLE work_run_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_run_id UUID NOT NULL REFERENCES work_runs(id) ON DELETE CASCADE,
    sequence BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (work_run_id, sequence)
);

CREATE INDEX idx_work_run_events_run_sequence ON work_run_events(work_run_id, sequence);
