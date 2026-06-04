use std::io::Write;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::paths;

fn default_harness() -> String {
    "host".to_owned()
}

fn default_image() -> String {
    crate::constants::DEFAULT_IMAGE.to_owned()
}

const fn default_poll_interval() -> u64 {
    15
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    #[serde(default = "default_harness")]
    pub harness: String,
    #[serde(default = "default_image")]
    pub image: String,
    #[serde(default)]
    pub log_format: Option<String>,
    #[serde(default)]
    pub debug: bool,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            harness: default_harness(),
            image: default_image(),
            log_format: None,
            debug: false,
            poll_interval_secs: default_poll_interval(),
        }
    }
}

pub fn load_config() -> anyhow::Result<WorkerConfig> {
    let path = paths::config_path()?;
    if !path.exists() {
        let _ = paths::ensure_vulcanum_dir()?;
        let config = WorkerConfig::default();
        save_config(&config)?;
        return Ok(config);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    serde_json::from_str(&data)
        .with_context(|| format!("failed to parse config from {}", path.display()))
}

pub fn save_config(config: &WorkerConfig) -> anyhow::Result<()> {
    let path = paths::config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    let mut file = std::fs::File::create(&path)
        .with_context(|| format!("failed to open {} for writing", path.display()))?;
    file.write_all(data.as_bytes())
        .with_context(|| format!("failed to write config to {}", path.display()))?;
    Ok(())
}
