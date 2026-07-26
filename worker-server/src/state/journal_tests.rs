mod lifecycle;
mod persistence;
mod validation;

use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

use crate::state::journal::{Journal, JournalInsert, JournalResultUpdate, JournalStatus};

fn journal_path() -> PathBuf {
    std::env::temp_dir().join(format!("vulcanum-journal-test-{}", Uuid::new_v4()))
}

fn open_journal_at(path: &Path) -> Journal {
    Journal::open(path).expect("should open journal")
}

fn open_journal() -> Journal {
    let path = journal_path();
    open_journal_at(&path)
}

fn insert_running_job(journal: &Journal, job_id: Uuid) {
    journal
        .insert_job(JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("should insert");
}
