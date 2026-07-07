use std::path::Path;
use std::sync::Mutex;

use anyhow::Context;
use rusqlite::Connection;

use crate::state::journal::Journal;

impl Journal {
    pub fn open(db_path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("failed to open journal at {}", db_path.display()))?;
        conn.execute_batch("PRAGMA journal_mode = WAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS job_journal (
                job_id TEXT PRIMARY KEY,
                workdir TEXT NOT NULL,
                container_name TEXT,
                harness_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'running',
                started_at TEXT NOT NULL,
                finished_at TEXT,
                exit_code INTEGER,
                tokens_used INTEGER,
                input_tokens INTEGER,
                output_tokens INTEGER,
                cache_read_tokens INTEGER,
                cache_write_tokens INTEGER,
                pr_url TEXT,
                duration_ms INTEGER,
                review_url TEXT,
                review_body TEXT,
                review_already_exists INTEGER NOT NULL DEFAULT 0,
                error_message TEXT,
                turn_count INTEGER NOT NULL DEFAULT 0,
                session_id TEXT,
                max_turns INTEGER NOT NULL DEFAULT 1,
                agent_backend TEXT NOT NULL DEFAULT 'opencode',
                agent_session_path TEXT,
                agent_config_dir TEXT,
                agent_state_dir TEXT,
                agent_transport TEXT,
                agent_pid INTEGER
            )",
        )?;

        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN turn_count INTEGER NOT NULL DEFAULT 0",
        )?;
        apply_column_migration(&conn, "ALTER TABLE job_journal ADD COLUMN session_id TEXT")?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN max_turns INTEGER NOT NULL DEFAULT 1",
        )?;
        apply_column_migration(&conn, "ALTER TABLE job_journal ADD COLUMN host_pid INTEGER")?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN host_port INTEGER",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN input_tokens INTEGER",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN output_tokens INTEGER",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN cache_read_tokens INTEGER",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN cache_write_tokens INTEGER",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_backend TEXT NOT NULL DEFAULT 'opencode'",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_session_path TEXT",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_config_dir TEXT",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_state_dir TEXT",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_transport TEXT",
        )?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN agent_pid INTEGER",
        )?;
        apply_column_migration(&conn, "ALTER TABLE job_journal ADD COLUMN review_url TEXT")?;
        apply_column_migration(&conn, "ALTER TABLE job_journal ADD COLUMN review_body TEXT")?;
        apply_column_migration(
            &conn,
            "ALTER TABLE job_journal ADD COLUMN review_already_exists INTEGER NOT NULL DEFAULT 0",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn apply_column_migration(conn: &Connection, sql: &str) -> anyhow::Result<()> {
    match conn.execute_batch(sql) {
        Ok(()) => Ok(()),
        Err(err) if is_duplicate_column_error(&err) => Ok(()),
        Err(err) => Err(err).with_context(|| format!("failed to apply journal migration: {sql}")),
    }
}

fn is_duplicate_column_error(err: &rusqlite::Error) -> bool {
    match err {
        rusqlite::Error::SqliteFailure(_, Some(message)) => {
            message.contains("duplicate column name")
        }
        _ => false,
    }
}
