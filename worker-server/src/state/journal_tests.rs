use chrono::Utc;
use uuid::Uuid;

use crate::state::journal::Journal;

#[test]
fn journal_persists_host_info() {
    let db_path = std::env::temp_dir().join(format!(
        "vulcanum-test-journal-host-info-{}.db",
        Uuid::new_v4()
    ));
    let journal = Journal::open(&db_path).expect("open journal");
    let job_id = Uuid::new_v4();

    journal
        .insert_job(
            job_id,
            "/tmp/vulcanum-work-test",
            None,
            "host",
            Utc::now(),
            1,
        )
        .expect("insert job");

    journal
        .set_host_info(job_id, 12345, 5555)
        .expect("set host info");

    let running = journal.list_running().expect("list running");
    let entry = running
        .into_iter()
        .find(|e| e.job_id == job_id)
        .expect("entry found");

    assert_eq!(entry.host_pid, Some(12345));
    assert_eq!(entry.host_port, Some(5555));

    let _ = std::fs::remove_file(&db_path);
}
