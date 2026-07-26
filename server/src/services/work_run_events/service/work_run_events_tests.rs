mod append;
mod listing;

use std::sync::Arc;

use chrono::{TimeZone, Utc};

use crate::db::work_run_events::WorkRunEventsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_run_events::errors::WorkRunEventsError;
use crate::services::dispatcher::cancel_store::{CancelStore, InMemoryCancelStore};
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::test_helpers;
use vulcanum_shared::api::wire::WireEvent;

fn make_wire_event(seq: u64, event_type: &str) -> WireEvent {
    WireEvent {
        sequence: seq,
        event_type: event_type.to_owned(),
        payload: serde_json::json!({"i": seq}),
        occurred_at: Utc::now(),
    }
}

fn build_service(pool: sqlx::PgPool) -> (WorkRunEventsService, Arc<InMemoryCancelStore>) {
    let cancel = Arc::new(InMemoryCancelStore::new());
    let svc = WorkRunEventsService::new(
        WorkRunEventsRepository::new(),
        WorkRunsRepository::new(),
        cancel.clone(),
        pool,
    );
    (svc, cancel)
}
