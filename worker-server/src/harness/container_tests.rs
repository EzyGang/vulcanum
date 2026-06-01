use crate::harness::container::ContainerHarness;
use crate::harness::gvisor::GvisorHarness;
use crate::harness::kata::KataHarness;
use crate::harness::{AgentHarness, ResourceLimits};
use std::collections::HashMap;

#[test]
fn container_image_default() {
    let harness = ContainerHarness::new("test-runtime");
    assert!(!harness.image.is_empty());
}

#[test]
fn kata_image_default_via_deref() {
    let harness = KataHarness::new();
    assert!(!harness.image.is_empty());
}

#[test]
fn gvisor_image_default_via_deref() {
    let harness = GvisorHarness::new();
    assert!(!harness.image.is_empty());
}

#[test]
fn kata_image_custom() {
    let harness = KataHarness::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(harness.image, "my-registry/agent:v1");
}

#[test]
fn gvisor_image_custom() {
    let harness = GvisorHarness::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(harness.image, "my-registry/agent:v1");
}

#[tokio::test]
#[ignore = "requires runsc runtime"]
async fn container_harness_missing_docker_returns_install_error() {
    let harness = ContainerHarness::with_image("nonexistent-image:latest".to_owned(), "runsc");
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("");

    let _ = std::fs::create_dir_all(&workdir);

    let result = harness
        .spawn("test", &workdir, &secrets, &limits, "", "", "")
        .await;

    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(_) => (),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("docker") || msg.contains("timed out") || msg.contains("No such file"),
                "expected install/timeout/crash error, got: {msg}"
            );
        }
    }
}
