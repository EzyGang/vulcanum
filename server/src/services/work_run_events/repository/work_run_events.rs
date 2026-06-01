use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::repository::WorkRunEventsRepository;

pub struct InsertEventParams {
    pub sequence: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub struct InsertBatchResult {
    pub accepted: u64,
    pub next_expected_sequence: i64,
}

impl WorkRunEventsRepository {
    pub async fn max_sequence<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        work_run_id: Uuid,
    ) -> Result<i64, WorkRunEventsError> {
        let row = sqlx::query!(
            r#"SELECT COALESCE(MAX(sequence), 0) AS "max!: i64" FROM work_run_events WHERE work_run_id = $1"#,
            work_run_id,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunEventsError::from)?;

        Ok(row.max)
    }

    /// Inserts a batch of events. Events must be strictly increasing in
    /// `sequence` and the first must be exactly `current_max + 1`. Rejection
    /// returns `OutOfOrderSequence` with the correct next sequence.
    ///
    /// Uses a transaction internally so partial inserts never persist.
    pub async fn insert_batch(
        &self,
        pool: &sqlx::PgPool,
        work_run_id: Uuid,
        events: &[InsertEventParams],
    ) -> Result<InsertBatchResult, WorkRunEventsError> {
        let mut tx = pool.begin().await.map_err(WorkRunEventsError::Database)?;

        let result = self.insert_batch_in_tx(&mut tx, work_run_id, events).await;

        match result {
            Ok(r) => {
                tx.commit().await.map_err(WorkRunEventsError::Database)?;
                Ok(r)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    async fn insert_batch_in_tx(
        &self,
        conn: &mut PgConnection,
        work_run_id: Uuid,
        events: &[InsertEventParams],
    ) -> Result<InsertBatchResult, WorkRunEventsError> {
        if events.is_empty() {
            let max = self.max_sequence(&mut *conn, work_run_id).await?;
            return Ok(InsertBatchResult {
                accepted: 0,
                next_expected_sequence: max + 1,
            });
        }

        for pair in events.windows(2) {
            if pair[1].sequence <= pair[0].sequence {
                return Err(WorkRunEventsError::OutOfOrderSequence {
                    next_expected_sequence: pair[0].sequence + 1,
                });
            }
        }

        let first_seq = events[0].sequence;
        let last_seq = events[events.len() - 1].sequence;

        let max = self.max_sequence(&mut *conn, work_run_id).await?;
        if first_seq != max + 1 {
            return Err(WorkRunEventsError::OutOfOrderSequence {
                next_expected_sequence: max + 1,
            });
        }

        for event in events {
            sqlx::query!(
                r#"INSERT INTO work_run_events (work_run_id, sequence, event_type, payload)
                   VALUES ($1, $2, $3, $4)"#,
                work_run_id,
                event.sequence,
                &event.event_type,
                event.payload,
            )
            .execute(&mut *conn)
            .await
            .map_err(map_insert_error)?;
        }

        Ok(InsertBatchResult {
            accepted: events.len() as u64,
            next_expected_sequence: last_seq + 1,
        })
    }

    pub async fn find_after<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        work_run_id: Uuid,
        after_sequence: i64,
        limit: i64,
    ) -> Result<Vec<WorkRunEvent>, WorkRunEventsError> {
        sqlx::query_as!(
            WorkRunEvent,
            r#"SELECT id, work_run_id, sequence, event_type, payload,
               created_at as "created_at!: DateTime<Utc>"
               FROM work_run_events
               WHERE work_run_id = $1 AND sequence > $2
               ORDER BY sequence ASC
               LIMIT $3"#,
            work_run_id,
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
               created_at as "created_at!: DateTime<Utc>"
               FROM (
                 SELECT * FROM work_run_events
                 WHERE work_run_id = $1
                 ORDER BY sequence DESC
                 LIMIT $2
               ) sub
               ORDER BY sequence ASC"#,
            work_run_id,
            limit,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunEventsError::from)
    }
}

fn map_insert_error(e: sqlx::Error) -> WorkRunEventsError {
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.code().as_deref() == Some("23505") {
            return WorkRunEventsError::OutOfOrderSequence {
                next_expected_sequence: 0,
            };
        }
    }
    WorkRunEventsError::Database(e)
}
