use crate::api::wire::WorkerCapabilities;

#[test]
fn worker_capabilities_default_to_no_isolation_backends() {
    let capabilities: WorkerCapabilities = serde_json::from_value(serde_json::json!({}))
        .expect("empty capabilities should deserialize");

    assert!(capabilities.isolation_backends.is_empty());
}

#[test]
fn worker_capabilities_ignore_legacy_agent_backends() {
    let capabilities: WorkerCapabilities = serde_json::from_value(serde_json::json!({
        "agent_backends": ["opencode", "omp_rpc"],
        "isolation_backends": ["host", "docker"]
    }))
    .expect("legacy capabilities should deserialize");

    assert_eq!(
        capabilities.isolation_backends,
        vec!["host".to_owned(), "docker".to_owned()]
    );
    assert_eq!(
        serde_json::to_value(capabilities).expect("capabilities should serialize"),
        serde_json::json!({
            "isolation_backends": ["host", "docker"]
        })
    );
}
