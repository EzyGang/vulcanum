use uuid::Uuid;

use vulcanum_shared::runtime::agent::RunningSession;

use crate::storage::messages::MessageStore;

pub(super) async fn save_session_messages(
    job_id: Uuid,
    running_session: &mut Box<dyn RunningSession>,
) {
    if let Some(sid) = running_session.session_id() {
        match running_session.export_messages().await {
            Ok(Some(messages)) => match MessageStore::new() {
                Ok(store) => {
                    let _ = store.save(job_id, sid, &messages);
                }
                Err(e) => {
                    tracing::warn!(work_run_id = %job_id, error = %e, "failed to create message store");
                }
            },
            Ok(None) => (),
            Err(e) => {
                tracing::warn!(work_run_id = %job_id, error = %e, "failed to export session messages");
            }
        }
    }
}
