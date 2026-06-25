use std::sync::Arc;

use sqlx::PgPool;

use crate::db::dispatcher::DispatchRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::db::workers::WorkersRepository;
use crate::models::dispatcher::errors::DispatchError;
use crate::services::dispatcher::dispatch_store::DispatchStore;

const ORPHAN_THRESHOLD_SECS: i64 = 120;

#[derive(Clone)]
pub struct DispatcherService {
    dispatch_repo: DispatchRepository,
    workers_repo: WorkersRepository,
    work_runs_repo: WorkRunsRepository,
    db: PgPool,
    dispatch_store: Arc<dyn DispatchStore>,
    stale_threshold: chrono::Duration,
    stalled_running_threshold_secs: i64,
}

impl DispatcherService {
    pub fn new(
        dispatch_repo: DispatchRepository,
        workers_repo: WorkersRepository,
        work_runs_repo: WorkRunsRepository,
        db: PgPool,
        dispatch_store: Arc<dyn DispatchStore>,
        stale_threshold_secs: u64,
        stalled_running_threshold_secs: u64,
    ) -> Self {
        Self {
            dispatch_repo,
            workers_repo,
            work_runs_repo,
            db,
            dispatch_store,
            stale_threshold: chrono::Duration::seconds(stale_threshold_secs as i64),
            stalled_running_threshold_secs: stalled_running_threshold_secs as i64,
        }
    }

    pub async fn dispatch_once(&self) -> Result<DispatchSummary, DispatchError> {
        let disconnected = self
            .workers_repo
            .mark_stale_disconnected(&self.db, self.stale_threshold)
            .await?;

        let orphaned_global = self
            .work_runs_repo
            .reset_orphaned_dispatched(&self.db, ORPHAN_THRESHOLD_SECS)
            .await
            .unwrap_or(0);

        let orphaned_no_worker = self
            .work_runs_repo
            .reset_orphaned_worker_runs(&self.db)
            .await
            .unwrap_or(0);

        let stalled_running = self
            .work_runs_repo
            .reset_stalled_running(&self.db, self.stalled_running_threshold_secs)
            .await
            .unwrap_or(0);

        let workers = self.dispatch_repo.find_available_workers(&self.db).await?;
        let pending = self.dispatch_repo.find_pending_unassigned(&self.db).await?;

        if workers.is_empty() || pending.is_empty() {
            tracing::debug!(
                workers_available = workers.len(),
                pending_jobs = pending.len(),
                workers = ?workers
                    .iter()
                    .map(|worker| {
                        format!(
                            "{}:{}:{:?}:{}/{}",
                            worker.id,
                            worker.team_id,
                            worker.status,
                            worker.active_jobs,
                            worker.max_concurrent_jobs
                        )
                    })
                    .collect::<Vec<String>>(),
                pending = ?pending
                    .iter()
                    .map(|work_run| {
                        format!(
                            "{}:{}:{}:{}",
                            work_run.id,
                            work_run.team_id,
                            work_run.project_config_id,
                            work_run.external_task_ref
                        )
                    })
                    .collect::<Vec<String>>(),
                "dispatch skipped because no workers or pending jobs are available",
            );

            return Ok(DispatchSummary {
                workers_available: workers.len(),
                pending_jobs: pending.len(),
                dispatched: 0,
                disconnected,
                orphaned: orphaned_global + orphaned_no_worker + stalled_running,
            });
        }

        let mut dispatched = 0usize;

        let mut used_workers: std::collections::HashSet<uuid::Uuid> =
            std::collections::HashSet::new();

        for work_run in &pending {
            let worker = match workers.iter().find(|worker| {
                worker.team_id == work_run.team_id && !used_workers.contains(&worker.id)
            }) {
                Some(worker) => worker,
                None => {
                    tracing::debug!(
                        work_run_id = %work_run.id,
                        work_run_team_id = %work_run.team_id,
                        used_workers = ?used_workers,
                        available_worker_teams = ?workers
                            .iter()
                            .map(|worker| worker.team_id)
                            .collect::<Vec<uuid::Uuid>>(),
                        "no available worker for pending work_run team",
                    );
                    continue;
                }
            };
            let mut tx = self.db.begin().await.map_err(DispatchError::Database)?;

            match self
                .dispatch_repo
                .dispatch_to_worker(&mut *tx, work_run.id, worker.id)
                .await
            {
                Ok(Some(_)) => {
                    if let Err(e) = self
                        .dispatch_repo
                        .increment_worker_jobs(&mut *tx, worker.id)
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            worker_id = %worker.id,
                            "failed to increment worker jobs"
                        );
                    }

                    if let Err(e) = tx.commit().await {
                        tracing::error!(
                            error = %e,
                            worker_id = %worker.id,
                            work_run_id = %work_run.id,
                            "failed to commit dispatch transaction"
                        );
                        continue;
                    }

                    if let Err(e) = self
                        .dispatch_store
                        .set_dispatched(worker.id, work_run.id)
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            worker_id = %worker.id,
                            work_run_id = %work_run.id,
                            "failed to set dispatch flag in redis"
                        );
                    }

                    dispatched += 1;
                    used_workers.insert(worker.id);
                }
                Ok(None) => {
                    let _ = tx.rollback().await;
                    tracing::debug!(
                        work_run_id = %work_run.id,
                        "work_run already claimed, skipping"
                    );
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    tracing::error!(
                        error = %e,
                        work_run_id = %work_run.id,
                        worker_id = %worker.id,
                        "failed to dispatch work_run"
                    );
                }
            }
        }

        tracing::info!(
            dispatched,
            workers_available = workers.len(),
            pending_jobs = pending.len(),
            disconnected,
            orphaned = orphaned_global + orphaned_no_worker + stalled_running,
            "dispatch cycle complete"
        );

        Ok(DispatchSummary {
            workers_available: workers.len(),
            pending_jobs: pending.len(),
            dispatched,
            disconnected,
            orphaned: orphaned_global + orphaned_no_worker + stalled_running,
        })
    }
}

pub struct DispatchSummary {
    pub workers_available: usize,
    pub pending_jobs: usize,
    pub dispatched: usize,
    pub disconnected: u64,
    pub orphaned: u64,
}
