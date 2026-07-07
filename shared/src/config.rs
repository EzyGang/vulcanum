use std::fmt;
use std::io::Write;
use std::str::FromStr;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::api_types::AgentBackend;
use crate::paths;

fn default_harness() -> String {
    IsolationBackend::Host.as_str().to_owned()
}

fn default_image() -> String {
    crate::constants::DEFAULT_IMAGE.to_owned()
}

const fn default_poll_interval() -> u64 {
    30
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationBackend {
    Host,
    Docker,
    Kata,
}

impl IsolationBackend {
    pub const VALUES: &'static [&'static str] = &["host", "docker", "kata"];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Docker => "docker",
            Self::Kata => "kata",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnknownIsolationBackend {
    value: String,
}

impl UnknownIsolationBackend {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for UnknownIsolationBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown isolation backend {:?}; expected one of: {}",
            self.value,
            IsolationBackend::VALUES.join(", ")
        )
    }
}

impl std::error::Error for UnknownIsolationBackend {}

impl FromStr for IsolationBackend {
    type Err = UnknownIsolationBackend;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "host" => Ok(Self::Host),
            "docker" => Ok(Self::Docker),
            "kata" => Ok(Self::Kata),
            _ => Err(UnknownIsolationBackend {
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    #[serde(default = "default_harness")]
    pub harness: String,
    #[serde(default = "default_image")]
    pub image: String,
    #[serde(default)]
    pub agent_backend: AgentBackend,
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
            agent_backend: AgentBackend::OpenCode,
            log_format: None,
            debug: false,
            poll_interval_secs: default_poll_interval(),
        }
    }
}

impl WorkerConfig {
    pub fn isolation_backend(&self) -> Result<IsolationBackend, UnknownIsolationBackend> {
        self.harness.parse()
    }
}

pub fn load_config() -> anyhow::Result<WorkerConfig> {
    match load_config_if_exists()? {
        Some(config) => Ok(config),
        None => {
            let _ = paths::ensure_vulcanum_dir()?;
            let config = WorkerConfig::default();
            save_config(&config)?;
            Ok(config)
        }
    }
}

pub fn load_config_if_exists() -> anyhow::Result<Option<WorkerConfig>> {
    let path = paths::config_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    let config = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse config from {}", path.display()))?;
    Ok(Some(config))
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
