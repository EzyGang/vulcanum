use crate::api::wire::{AgentBackend, WorkerCapabilities};

#[test]
fn worker_capabilities_default_to_opencode() {
    let capabilities: WorkerCapabilities = serde_json::from_value(serde_json::json!({}))
        .expect("empty capabilities should deserialize");

    assert_eq!(capabilities.agent_backends, vec![AgentBackend::OpenCode]);
    assert!(capabilities.supports_agent_backend(AgentBackend::OpenCode));
    assert!(!capabilities.supports_agent_backend(AgentBackend::OmpRpc));
}

#[test]
fn worker_capabilities_use_backend_wire_names() {
    let capabilities = WorkerCapabilities {
        agent_backends: vec![AgentBackend::OpenCode, AgentBackend::OmpRpc],
        isolation_backends: vec!["host".to_owned(), "docker".to_owned()],
    };

    let encoded = serde_json::to_value(&capabilities).expect("capabilities should serialize");
    assert_eq!(
        encoded,
        serde_json::json!({
            "agent_backends": ["opencode", "omp_rpc"],
            "isolation_backends": ["host", "docker"]
        })
    );

    let decoded: WorkerCapabilities =
        serde_json::from_value(encoded).expect("capabilities should deserialize");
    assert_eq!(decoded, capabilities);
}
