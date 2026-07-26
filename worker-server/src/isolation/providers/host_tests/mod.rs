mod preparation;
mod runtime_and_cleanup;

use vulcanum_shared::api::wire::{
    AgentBackend, AgentConfigPayload, OpenCodeProviderConfig, WorkRunType,
};
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::isolation::providers::host::HostIsolation;

fn opencode_config(model: Option<&str>) -> AgentConfigPayload {
    AgentConfigPayload::OpenCode {
        providers: std::collections::HashMap::new(),
        model: model.map(str::to_owned),
        small_model: None,
        auth_content: None,
    }
}
