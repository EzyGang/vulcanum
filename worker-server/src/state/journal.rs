mod model;
mod row;
mod schema;

use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

pub use crate::state::journal::model::{
    JournalEntry, JournalInsert, JournalResultUpdate, JournalStatus,
};

pub struct Journal {
    conn: Mutex<Connection>,
}

impl Journal {
    pub fn insert_job(&self, job: JournalInsert<'_>) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "INSERT INTO job_journal (job_id, workdir, container_name, harness_type, status, started_at, max_turns, agent_backend)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                job.job_id.to_string(),
                job.workdir,
                job.container_name,
                job.harness_type,
                JournalStatus::Running.as_str(),
                job.started_at.to_rfc3339(),
                job.max_turns,
                job.agent_backend,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_id(&self, job_id: Uuid) -> anyhow::Result<Option<JournalEntry>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT job_id, workdir, container_name, harness_type, status, started_at,
                    finished_at, exit_code, tokens_used, input_tokens, output_tokens,
                    cache_read_tokens, cache_write_tokens, pr_url, duration_ms,
                    review_url, review_body, review_already_exists, error_message,
                    turn_count, session_id, max_turns, host_pid, host_port, agent_backend,
                    agent_session_path, agent_config_dir, agent_state_dir, agent_transport, agent_pid
             FROM job_journal WHERE job_id = ?1",
        )?;

        let mut rows = stmt.query_map([job_id.to_string()], row::journal_entry_from_row)?;
        match rows.next() {
            Some(entry) => entry.map(Some).map_err(Into::into),
            None => Ok(None),
        }
    }

    pub fn remove_job(&self, job_id: Uuid) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "DELETE FROM job_journal WHERE job_id = ?1",
            [job_id.to_string()],
        )?;
        Ok(())
    }

    pub fn update_result(&self, result: JournalResultUpdate<'_>) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET status = ?1, finished_at = ?2, exit_code = ?3, tokens_used = ?4,
             input_tokens = ?5, output_tokens = ?6, cache_read_tokens = ?7, cache_write_tokens = ?8,
             pr_url = ?9, duration_ms = ?10, review_url = ?11, review_body = ?12,
             review_already_exists = ?13
             WHERE job_id = ?14",
            rusqlite::params![
                result.status.as_str(),
                now,
                result.exit_code,
                result.tokens_used,
                result.input_tokens,
                result.output_tokens,
                result.cache_read_tokens,
                result.cache_write_tokens,
                result.pr_url,
                result.duration_ms,
                result.review_url,
                result.review_body,
                result.review_already_exists,
                result.job_id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn update_turn(&self, job_id: Uuid, turn_count: i32) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET turn_count = ?1 WHERE job_id = ?2",
            rusqlite::params![turn_count, job_id.to_string()],
        )?;
        Ok(())
    }

    pub fn set_session_id(&self, job_id: Uuid, session_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET session_id = ?1 WHERE job_id = ?2",
            rusqlite::params![session_id, job_id.to_string()],
        )?;
        Ok(())
    }

    pub fn set_host_info(&self, job_id: Uuid, host_pid: i64, host_port: i64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET host_pid = ?1, host_port = ?2 WHERE job_id = ?3",
            rusqlite::params![host_pid, host_port, job_id.to_string()],
        )?;
        Ok(())
    }

    pub fn set_agent_metadata(
        &self,
        job_id: Uuid,
        session_path: Option<&str>,
        config_dir: Option<&str>,
        state_dir: Option<&str>,
        transport: Option<&str>,
        pid: Option<i64>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET agent_session_path = ?1, agent_config_dir = ?2,
             agent_state_dir = ?3, agent_transport = ?4, agent_pid = ?5 WHERE job_id = ?6",
            rusqlite::params![
                session_path,
                config_dir,
                state_dir,
                transport,
                pid,
                job_id.to_string(),
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn mark_lost(&self, job_id: Uuid, error_message: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET status = ?1, finished_at = ?2, error_message = ?3
             WHERE job_id = ?4",
            rusqlite::params![
                JournalStatus::Lost.as_str(),
                now,
                error_message,
                job_id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn mark_submitted(&self, job_id: Uuid) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE job_journal SET status = ?1 WHERE job_id = ?2",
            rusqlite::params![JournalStatus::Submitted.as_str(), job_id.to_string()],
        )?;
        Ok(())
    }

    pub fn list_running(&self) -> anyhow::Result<Vec<JournalEntry>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT job_id, workdir, container_name, harness_type, status, started_at,
                    finished_at, exit_code, tokens_used, input_tokens, output_tokens,
                    cache_read_tokens, cache_write_tokens, pr_url, duration_ms,
                    review_url, review_body, review_already_exists, error_message,
                    turn_count, session_id, max_turns, host_pid, host_port, agent_backend,
                    agent_session_path, agent_config_dir, agent_state_dir, agent_transport, agent_pid
             FROM job_journal WHERE status = 'running'",
        )?;

        let rows = stmt
            .query_map([], row::journal_entry_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows)
    }
}
