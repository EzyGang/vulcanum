use std::future::Future;
use std::time::Duration;

use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;

#[cfg(not(test))]
const RESERVATION_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
#[cfg(test)]
const RESERVATION_HEARTBEAT_INTERVAL: Duration = Duration::from_millis(25);

impl WorkRunsService {
    pub(crate) async fn run_with_implementation_followup_heartbeat<F, T>(
        &self,
        project_config_id: Uuid,
        normalized_repo: &str,
        pr_number: i64,
        token: Uuid,
        operation: F,
    ) -> Result<T, WorkRunsError>
    where
        F: Future<Output = Result<T, WorkRunsError>>,
    {
        let first_heartbeat = tokio::time::Instant::now() + RESERVATION_HEARTBEAT_INTERVAL;
        let mut heartbeat =
            tokio::time::interval_at(first_heartbeat, RESERVATION_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        tokio::pin!(operation);

        loop {
            tokio::select! {
                result = &mut operation => return result,
                _ = heartbeat.tick() => {
                    let renewed = self.work_runs_repo
                        .renew_github_implementation_followup_ticket(
                            &self.db,
                            project_config_id,
                            normalized_repo,
                            pr_number,
                            token,
                        )
                        .await?;
                    if !renewed {
                        return Err(WorkRunsError::ImplementationFollowupPending);
                    }
                }
            }
        }
    }
}
