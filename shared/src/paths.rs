use std::path::PathBuf;

use anyhow::Context;

pub fn vulcanum_dir() -> anyhow::Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("failed to find home directory")?
        .join(".vulcanum");
    Ok(dir)
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    vulcanum_dir().map(|d| d.join("config.json"))
}

pub fn state_path() -> anyhow::Result<PathBuf> {
    vulcanum_dir().map(|d| d.join("worker.json"))
}

pub fn db_path() -> anyhow::Result<PathBuf> {
    vulcanum_dir().map(|d| d.join("worker.db"))
}

pub fn sessions_dir() -> anyhow::Result<PathBuf> {
    vulcanum_dir().map(|d| d.join("sessions"))
}

pub fn session_messages_dir(work_run_id: &str) -> anyhow::Result<PathBuf> {
    sessions_dir().map(|d| d.join(work_run_id))
}

pub fn ensure_vulcanum_dir() -> anyhow::Result<PathBuf> {
    let dir = vulcanum_dir()?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create vulcanum dir {}", dir.display()))?;
    Ok(dir)
}
