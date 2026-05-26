use std::sync::Arc;

use sqlx::PgPool;

use crate::services::dispatcher::errors::DispatchError;
use crate::services::dispatcher::flag_store::DispatchStore;
use crate::services::dispatcher::repository::DispatchRepository;
use crate::services::workers::repository::WorkersRepository;

#[derive(Clone)]
pub struct DispatcherService {
    dispatch_repo: DispatchRepository,
    workers_repo: WorkersRepository,
    db: PgPool,
    dispatch_store: Arc<dyn DispatchStore>,
    stale_threshold: chrono::Duration,
}

impl DispatcherService {
    pub fn new(
        dispatch_repo: DispatchRepository,
        workers_repo: WorkersRepository,
        db: PgPool,
        dispatch_store: Arc<dyn DispatchStore>,
        stale_threshold_secs: u64,
    ) -> Self {
        Self {
            dispatch_repo,
            workers_repo,
            db,
            dispatch_store,
            stale_threshold: chrono::Duration::seconds(stale_threshold_secs as i64),
        }
    }

    pub async fn dispatch_once(&self) -> Result<DispatchSummary, DispatchError> {
        let disconnected = self
            .workers_repo
            .mark_stale_disconnected(&self.db, self.stale_threshold)
            .await?;

        let workers = self.dispatch_repo.find_idle_workers(&self.db).await?;
        let pending = self.dispatch_repo.find_pending_unassigned(&self.db).await?;

        if workers.is_empty() || pending.is_empty() {
            return Ok(DispatchSummary {
                workers_available: workers.len(),
                pending_jobs: pending.len(),
                dispatched: 0,
                disconnected,
            });
        }

        let mut dispatched = 0usize;

        for (worker, work_run) in workers.iter().zip(pending.iter()) {
            match self
                .dispatch_repo
                .dispatch_to_worker(&self.db, work_run.id, worker.id)
                .await
            {
                Ok(Some(_)) => {
                    if let Err(e) = self
                        .dispatch_store
                        .set_dispatched(worker.id, work_run.id)
                        .await
                    {
                        tracing::warn!(
                            worker_id = worker.id.to_string().as_str(),
                            work_run_id = work_run.id.to_string().as_str(),
                            "failed to set dispatch flag in redis: {e}",
                        );
                    }

                    if let Err(e) = self
                        .dispatch_repo
                        .set_worker_busy(&self.db, worker.id)
                        .await
                    {
                        tracing::warn!(
                            worker_id = worker.id.to_string().as_str(),
                            "failed to set worker status to busy: {e}",
                        );
                    }

                    dispatched += 1;
                }
                Ok(None) => {
                    tracing::debug!(
                        work_run_id = work_run.id.to_string().as_str(),
                        "work_run already claimed, skipping",
                    );
                }
                Err(e) => {
                    tracing::error!(
                        work_run_id = work_run.id.to_string().as_str(),
                        worker_id = worker.id.to_string().as_str(),
                        "failed to dispatch work_run: {e}",
                    );
                }
            }
        }

        tracing::info!(
            dispatched,
            workers_available = workers.len(),
            pending_jobs = pending.len(),
            disconnected,
            "dispatch cycle complete: {dispatched} jobs dispatched",
        );

        Ok(DispatchSummary {
            workers_available: workers.len(),
            pending_jobs: pending.len(),
            dispatched,
            disconnected,
        })
    }
}

pub struct DispatchSummary {
    pub workers_available: usize,
    pub pending_jobs: usize,
    pub dispatched: usize,
    pub disconnected: u64,
}
