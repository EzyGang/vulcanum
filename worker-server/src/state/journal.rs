use std::path::Path;
use std::sync::Mutex;

use anyhow::Context;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum JournalStatus {
    Running,
    Completed,
    Failed,
    Lost,
    Submitted,
}

impl JournalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Lost => "lost",
            Self::Submitted => "submitted",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "lost" => Some(Self::Lost),
            "submitted" => Some(Self::Submitted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct JournalEntry {
    pub job_id: Uuid,
    pub workdir: String,
    pub container_name: Option<String>,
    pub harness_type: String,
    pub status: JournalStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub tokens_used: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub pr_url: Option<String>,
    pub duration_ms: Option<i64>,
    pub review_url: Option<String>,
    pub review_body: Option<String>,
    pub review_already_exists: bool,
    pub error_message: Option<String>,
    pub turn_count: Option<i32>,
    pub session_id: Option<String>,
    pub max_turns: Option<i32>,
    pub host_pid: Option<i64>,
    pub host_port: Option<i64>,
    pub agent_backend: Option<String>,
    pub agent_session_path: Option<String>,
    pub agent_config_dir: Option<String>,
    pub agent_state_dir: Option<String>,
    pub agent_transport: Option<String>,
    pub agent_pid: Option<i64>,
}

pub struct JournalResultUpdate<'a> {
    pub job_id: Uuid,
    pub exit_code: i32,
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub pr_url: Option<&'a str>,
    pub duration_ms: i64,
    pub review_url: Option<&'a str>,
    pub review_body: Option<&'a str>,
    pub review_already_exists: bool,
    pub status: JournalStatus,
}

pub struct JournalInsert<'a> {
    pub job_id: Uuid,
    pub workdir: &'a str,
    pub container_name: Option<&'a str>,
    pub harness_type: &'a str,
    pub started_at: DateTime<Utc>,
    pub max_turns: i32,
    pub agent_backend: &'a str,
}

pub struct Journal {
    conn: Mutex<Connection>,
}

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

        let _ = conn.execute_batch(
            "ALTER TABLE job_journal ADD COLUMN turn_count INTEGER NOT NULL DEFAULT 0",
        );
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN session_id TEXT");
        let _ = conn.execute_batch(
            "ALTER TABLE job_journal ADD COLUMN max_turns INTEGER NOT NULL DEFAULT 1",
        );
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN host_pid INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN host_port INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN input_tokens INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN output_tokens INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN cache_read_tokens INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN cache_write_tokens INTEGER");
        let _ = conn.execute_batch(
            "ALTER TABLE job_journal ADD COLUMN agent_backend TEXT NOT NULL DEFAULT 'opencode'",
        );
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN agent_session_path TEXT");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN agent_config_dir TEXT");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN agent_state_dir TEXT");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN agent_transport TEXT");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN agent_pid INTEGER");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN review_url TEXT");
        let _ = conn.execute_batch("ALTER TABLE job_journal ADD COLUMN review_body TEXT");
        let _ = conn.execute_batch(
            "ALTER TABLE job_journal ADD COLUMN review_already_exists INTEGER NOT NULL DEFAULT 0",
        );

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

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

        let mut rows = stmt.query_map([job_id.to_string()], Self::journal_entry_from_row)?;
        match rows.next() {
            Some(entry) => entry.map(Some).map_err(Into::into),
            None => Ok(None),
        }
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
                result.pr_url.unwrap_or(""),
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
            .query_map([], Self::journal_entry_from_row)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }

    fn journal_entry_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JournalEntry> {
        let started_at: String = row.get(5)?;
        let finished_at: Option<String> = row.get(6)?;
        Ok(JournalEntry {
            job_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            workdir: row.get(1)?,
            container_name: row.get(2)?,
            harness_type: row.get(3)?,
            status: JournalStatus::from_str(&row.get::<_, String>(4)?)
                .unwrap_or(JournalStatus::Lost),
            started_at: DateTime::parse_from_rfc3339(&started_at)
                .map(|d| d.to_utc())
                .unwrap_or_default(),
            finished_at: finished_at
                .and_then(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.to_utc()).ok()),
            exit_code: row.get(7)?,
            tokens_used: row.get(8)?,
            input_tokens: row.get(9)?,
            output_tokens: row.get(10)?,
            cache_read_tokens: row.get(11)?,
            cache_write_tokens: row.get(12)?,
            pr_url: row.get(13)?,
            duration_ms: row.get(14)?,
            review_url: row.get(15)?,
            review_body: row.get(16)?,
            review_already_exists: row.get(17)?,
            error_message: row.get(18)?,
            turn_count: row.get(19)?,
            session_id: row.get(20)?,
            max_turns: row.get(21)?,
            host_pid: row.get(22)?,
            host_port: row.get(23)?,
            agent_backend: row.get(24)?,
            agent_session_path: row.get(25)?,
            agent_config_dir: row.get(26)?,
            agent_state_dir: row.get(27)?,
            agent_transport: row.get(28)?,
            agent_pid: row.get(29)?,
        })
    }
}
