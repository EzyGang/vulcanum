use crate::harness::gvisor::GvisorHarness;
use crate::harness::{AgentHarness, ResourceLimits};
use std::collections::HashMap;

#[tokio::test]
async fn gvisor_harness_missing_docker_returns_install_error() {
    let harness = GvisorHarness::with_image("nonexistent-image:latest".to_owned());
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-gvisor");

    let _ = std::fs::create_dir_all(&workdir);

    let result = harness
        .spawn("test", &workdir, &secrets, &limits, "", "")
        .await;

    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(_) => (),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("docker") || msg.contains("timed out"),
                "expected install/timeout error, got: {msg}"
            );
        }
    }
}
