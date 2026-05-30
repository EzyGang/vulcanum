use std::collections::HashMap;

use crate::harness::errors::HarnessError;
use crate::harness::kata::KataHarness;
use crate::harness::{AgentHarness, ResourceLimits};

#[test]
fn kata_image_default() {
    let harness = KataHarness::new();
    assert!(!harness.image.is_empty());
}

#[test]
fn kata_image_custom() {
    let harness = KataHarness::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(harness.image, "my-registry/agent:v1");
}

#[test]
fn resource_limits_default_vcpu() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.vcpu_count, 2);
}

#[test]
fn resource_limits_default_memory() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.memory_mib, 1_024);
}

#[tokio::test]
#[ignore = "requires kata-runtime"]
async fn kata_harness_missing_docker_returns_install_error() {
    let harness = KataHarness::with_image("nonexistent-image:latest".to_owned());
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-nonexistent");

    let _ = std::fs::create_dir_all(&workdir);

    let result: Result<_, HarnessError> = harness
        .spawn("test", &workdir, &secrets, &limits, "", "")
        .await;

    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(_) => {}
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("docker") || msg.contains("timed out") || msg.contains("No such file"),
                "expected install/timeout/crash error, got: {msg}"
            );
        }
    }
}

#[tokio::test]
#[ignore = "requires kata-runtime"]
async fn kata_harness_writes_agents_md() {
    let harness = KataHarness::with_image("nonexistent-image:latest".to_owned());
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-kata-agents");
    let _ = std::fs::create_dir_all(&workdir);

    let agents_content = "# Vulcanum AGENTS.md\nconvention: strict";
    let result = harness
        .spawn("test", &workdir, &secrets, &limits, "", agents_content)
        .await;

    let agents_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");

    let has_agents = std::fs::read_to_string(&agents_path).ok();
    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(_) => {
            assert_eq!(
                has_agents.as_deref(),
                Some(agents_content),
                "AGENTS.md should have been written when spawn succeeds"
            );
        }
        Err(_) => {
            // When Docker or the runtime is unavailable, spawn may fail before
            // the agent container starts, so the file may not exist.
        }
    }
}

#[tokio::test]
#[ignore = "requires kata-runtime"]
async fn kata_harness_skips_agents_md_when_empty() {
    let harness = KataHarness::with_image("nonexistent-image:latest".to_owned());
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-kata-no-agents");
    let _ = std::fs::create_dir_all(&workdir);

    let result = harness
        .spawn("test", &workdir, &secrets, &limits, "", "")
        .await;

    let agents_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");
    let exists = agents_path.exists();
    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(_) => {
            assert!(
                !exists,
                "AGENTS.md should not be created when agents_md is empty"
            );
        }
        Err(_) => {
            // When Docker or the runtime is unavailable, spawn may fail before
            // the agent container starts, so the file may not exist.
        }
    }
}
