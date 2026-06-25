use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::work_run_events::errors::WorkRunEventsError;
use crate::models::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::service::{map_work_runs_error, WorkRunEventsService};

const MAX_LIMIT: i64 = 500;
const DEFAULT_LIMIT: i64 = 100;

#[derive(Debug)]
pub struct ListResult {
    pub events: Vec<WorkRunEvent>,
    pub has_more: bool,
}

impl WorkRunEventsService {
    /// Worker-scoped listing. Verifies the worker owns the work run.
    pub async fn list_events(
        &self,
        work_run_id: Uuid,
        worker_id: Uuid,
        after_occurred_at: DateTime<Utc>,
        after_sequence: i64,
        limit: i64,
    ) -> Result<ListResult, WorkRunEventsError> {
        self.verify_work_run_owned(work_run_id, Some(worker_id))
            .await?;
        self.fetch_page(work_run_id, after_occurred_at, after_sequence, limit)
            .await
    }

    /// Instance-scoped listing. Skips ownership check.
    pub async fn list_events_admin(
        &self,
        work_run_id: Uuid,
        after_occurred_at: DateTime<Utc>,
        after_sequence: i64,
        limit: i64,
    ) -> Result<ListResult, WorkRunEventsError> {
        self.verify_work_run_owned(work_run_id, None).await?;
        self.fetch_page(work_run_id, after_occurred_at, after_sequence, limit)
            .await
    }

    pub async fn list_events_admin_for_team(
        &self,
        work_run_id: Uuid,
        team_id: Uuid,
        after_occurred_at: DateTime<Utc>,
        after_sequence: i64,
        limit: i64,
    ) -> Result<ListResult, WorkRunEventsError> {
        self.verify_work_run_team(work_run_id, team_id).await?;
        self.fetch_page(work_run_id, after_occurred_at, after_sequence, limit)
            .await
    }

    async fn verify_work_run_owned(
        &self,
        work_run_id: Uuid,
        worker_id: Option<Uuid>,
    ) -> Result<(), WorkRunEventsError> {
        let run = self
            .work_runs_repo
            .find_by_id(&self.db, work_run_id)
            .await
            .map_err(map_work_runs_error)?;

        if let Some(wid) = worker_id {
            if run.worker_id != Some(wid) {
                return Err(WorkRunEventsError::NotFound);
            }
        }

        Ok(())
    }

    async fn verify_work_run_team(
        &self,
        work_run_id: Uuid,
        team_id: Uuid,
    ) -> Result<(), WorkRunEventsError> {
        let run = self
            .work_runs_repo
            .find_by_id(&self.db, work_run_id)
            .await
            .map_err(map_work_runs_error)?;

        match run.team_id == team_id {
            true => Ok(()),
            false => Err(WorkRunEventsError::NotFound),
        }
    }

    /// Uses a composite (occurred_at, sequence) cursor for pagination.
    /// Passing the last seen event's timestamps prevents re-fetching
    /// events that share the same occurred_at timestamp.
    async fn fetch_page(
        &self,
        work_run_id: Uuid,
        after_occurred_at: DateTime<Utc>,
        after_sequence: i64,
        limit: i64,
    ) -> Result<ListResult, WorkRunEventsError> {
        let clamped = clamp_limit(limit);
        let events = self
            .repo
            .find_after(
                &self.db,
                work_run_id,
                after_occurred_at,
                after_sequence,
                clamped + 1,
            )
            .await?;

        let has_more = events.len() as i64 > clamped;
        let events = if has_more {
            events.into_iter().take(clamped as usize).collect()
        } else {
            events
        };

        Ok(ListResult { events, has_more })
    }
}

fn clamp_limit(limit: i64) -> i64 {
    if limit <= 0 {
        DEFAULT_LIMIT
    } else {
        limit.min(MAX_LIMIT)
    }
}
