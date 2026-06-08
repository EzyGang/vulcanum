use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::repository::WorkRunEventsRepository;

pub struct InsertEventParams {
    pub sequence: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct InsertBatchResult {
    pub accepted: u64,
}

impl WorkRunEventsRepository {
    pub async fn insert_batch(
        &self,
        pool: &sqlx::PgPool,
        work_run_id: Uuid,
        events: &[InsertEventParams],
    ) -> Result<InsertBatchResult, WorkRunEventsError> {
        let mut tx = pool.begin().await?;

        let mut accepted: u64 = 0;
        for event in events {
            let result = sqlx::query!(
                r#"INSERT INTO work_run_events (work_run_id, sequence, event_type, payload, occurred_at)
                   VALUES ($1, $2, $3, $4, $5)
                   ON CONFLICT (work_run_id, sequence) DO NOTHING"#,
                work_run_id,
                event.sequence,
                &event.event_type,
                event.payload,
                event.occurred_at,
            )
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() > 0 {
                accepted += 1;
            }
        }

        tx.commit().await?;

        Ok(InsertBatchResult { accepted })
    }

    pub async fn find_after<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        work_run_id: Uuid,
        after_occurred_at: DateTime<Utc>,
        after_sequence: i64,
        limit: i64,
    ) -> Result<Vec<WorkRunEvent>, WorkRunEventsError> {
        sqlx::query_as!(
            WorkRunEvent,
            r#"SELECT id, work_run_id, sequence, event_type, payload,
               created_at as "created_at!: DateTime<Utc>",
               occurred_at as "occurred_at!: DateTime<Utc>"
               FROM work_run_events
               WHERE work_run_id = $1
                 AND (occurred_at, sequence) > ($2, $3)
               ORDER BY occurred_at ASC, sequence ASC
               LIMIT $4"#,
            work_run_id,
            after_occurred_at,
            after_sequence,
            limit,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunEventsError::from)
    }

    pub async fn find_last_n<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        work_run_id: Uuid,
        limit: i64,
    ) -> Result<Vec<WorkRunEvent>, WorkRunEventsError> {
        sqlx::query_as!(
            WorkRunEvent,
            r#"SELECT id, work_run_id, sequence, event_type, payload,
               created_at as "created_at!: DateTime<Utc>",
               occurred_at as "occurred_at!: DateTime<Utc>"
               FROM (
                 SELECT * FROM work_run_events
                 WHERE work_run_id = $1
                 ORDER BY occurred_at DESC, sequence DESC
                 LIMIT $2
               ) sub
               ORDER BY occurred_at ASC, sequence ASC"#,
            work_run_id,
            limit,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunEventsError::from)
    }
}
