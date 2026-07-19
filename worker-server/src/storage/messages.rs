use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::Value;
use uuid::Uuid;

use vulcanum_shared::state::paths;

pub struct MessageStore;

impl MessageStore {
    pub fn new() -> anyhow::Result<Self> {
        paths::ensure_vulcanum_dir()?;
        Ok(Self)
    }

    pub fn save(&self, work_run_id: Uuid, session_id: &str, data: &Value) -> anyhow::Result<()> {
        let dir = paths::session_messages_dir(&work_run_id.to_string())?;
        let path = save_messages_to_dir(&dir, session_id, data)?;
        tracing::info!(
            work_run_id = %work_run_id,
            session_id = session_id,
            path = %path.display(),
            "stored session messages"
        );
        Ok(())
    }
}

fn save_messages_to_dir(dir: &Path, session_id: &str, data: &Value) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("failed to create session dir {}", dir.display()))?;
    let filename = format!("{}.json", encode_session_id(session_id));
    let path = dir.join(&filename);
    let temp_path = dir.join(format!("{filename}.tmp-{}", Uuid::new_v4()));
    let serialized = serde_json::to_string_pretty(data).context("failed to serialize messages")?;
    let mut temp_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .with_context(|| format!("failed to create temp message file {}", temp_path.display()))?;
    if let Err(err) = temp_file.write_all(serialized.as_bytes()) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err).with_context(|| {
            format!(
                "failed to write messages to temp file {}",
                temp_path.display()
            )
        });
    }
    if let Err(err) = temp_file.sync_all() {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err)
            .with_context(|| format!("failed to sync temp message file {}", temp_path.display()));
    }
    drop(temp_file);
    if let Err(err) = std::fs::rename(&temp_path, &path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err).with_context(|| {
            format!(
                "failed to atomically move messages from {} to {}",
                temp_path.display(),
                path.display()
            )
        });
    }
    Ok(path)
}

#[must_use]
pub(crate) fn encode_session_id(session_id: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(session_id.len() * 2);
    for byte in session_id.as_bytes() {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use crate::storage::messages::{encode_session_id, save_messages_to_dir};

    #[test]
    fn encode_session_id_hex_encodes_path_unsafe_bytes() {
        let session_id = "../windows\\session:one";

        let encoded = encode_session_id(session_id);

        assert_eq!(encoded, "2e2e2f77696e646f77735c73657373696f6e3a6f6e65");
        assert!(!encoded.chars().any(|ch| matches!(ch, '/' | '\\' | ':')));
    }

    #[test]
    fn save_messages_to_dir_writes_final_json_to_encoded_filename() {
        let dir =
            std::env::temp_dir().join(format!("vulcanum-message-store-test-{}", Uuid::new_v4()));
        let session_id = "../windows\\session:one";
        let data = json!({
            "messages": [
                {
                    "role": "user",
                    "content": "ship it"
                }
            ]
        });

        let path = save_messages_to_dir(&dir, session_id, &data).expect("messages should be saved");

        assert_eq!(path.parent(), Some(dir.as_path()));
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("2e2e2f77696e646f77735c73657373696f6e3a6f6e65.json")
        );
        let persisted = std::fs::read_to_string(&path).expect("message file should be readable");
        let persisted: serde_json::Value =
            serde_json::from_str(&persisted).expect("message file should contain JSON");
        assert_eq!(persisted, data);
        let entries = std::fs::read_dir(&dir)
            .expect("session dir should be readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("session dir entries should be readable");
        assert_eq!(entries.len(), 1);

        std::fs::remove_dir_all(&dir).expect("temp message dir should be removed");
    }
}
