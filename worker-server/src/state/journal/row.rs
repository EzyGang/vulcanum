use chrono::DateTime;
use rusqlite::types::Type;
use uuid::Uuid;

use crate::state::journal::model::{journal_status_from_str, JournalEntry};

pub(super) fn journal_entry_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JournalEntry> {
    let job_id = row.get::<_, String>(0)?;
    let status = row.get::<_, String>(4)?;
    let started_at = row.get::<_, String>(5)?;
    let finished_at = row.get::<_, Option<String>>(6)?;

    let job_id = Uuid::parse_str(&job_id)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(err)))?;
    let status = journal_status_from_str(&status)
        .ok_or_else(|| invalid_text_column(4, format!("invalid journal status {status:?}")))?;
    let started_at = DateTime::parse_from_rfc3339(&started_at)
        .map(|d| d.to_utc())
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(err)))?;
    let finished_at = finished_at
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|d| d.to_utc())
                .map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(err))
                })
        })
        .transpose()?;

    Ok(JournalEntry {
        job_id,
        workdir: row.get(1)?,
        container_name: row.get(2)?,
        harness_type: row.get(3)?,
        status,
        started_at,
        finished_at,
        exit_code: row.get(7)?,
        tokens_used: row.get(8)?,
        input_tokens: row.get(9)?,
        output_tokens: row.get(10)?,
        cache_read_tokens: row.get(11)?,
        cache_write_tokens: row.get(12)?,
        pr_url: normalize_optional_text(row.get(13)?),
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

fn invalid_text_column(column: usize, message: impl Into<String>) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message.into(),
        )),
    )
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}
