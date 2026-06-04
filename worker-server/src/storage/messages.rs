use std::path::PathBuf;

use anyhow::Context;
use serde_json::Value;
use uuid::Uuid;

use vulcanum_shared::paths;

pub struct MessageStore {
    base_dir: PathBuf,
}

impl MessageStore {
    pub fn new() -> anyhow::Result<Self> {
        let base_dir = paths::sessions_dir()?;
        std::fs::create_dir_all(&base_dir)
            .with_context(|| format!("failed to create sessions dir {}", base_dir.display()))?;
        Ok(Self { base_dir })
    }

    pub fn save(&self, work_run_id: Uuid, session_id: &str, data: &Value) -> anyhow::Result<()> {
        let dir = self.base_dir.join(work_run_id.to_string());
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create session dir {}", dir.display()))?;
        let path = dir.join(format!("{session_id}.json"));
        let serialized =
            serde_json::to_string_pretty(data).context("failed to serialize messages")?;
        std::fs::write(&path, serialized)
            .with_context(|| format!("failed to write messages to {}", path.display()))?;
        tracing::info!(
            work_run_id = %work_run_id,
            session_id = session_id,
            path = %path.display(),
            "stored session messages"
        );
        Ok(())
    }
}
