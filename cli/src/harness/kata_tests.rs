use std::collections::HashMap;

use crate::harness::errors::HarnessError;
use crate::harness::kata::KataHarness;
use crate::harness::parse::{parse_pr_url, parse_token_usage};
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

#[test]
fn parse_pr_url_from_docker_output() {
    let stdout = "Cloning repository...\nRunning opencode...\nhttps://github.com/vulcanum/repo/pull/99\nDone.";
    let url = parse_pr_url(stdout);
    assert_eq!(
        url,
        Some("https://github.com/vulcanum/repo/pull/99".to_owned())
    );
}

#[test]
fn parse_token_usage_from_docker_output() {
    let stdout = "Completed task\nTokens used: 5678\nPR submitted.";
    let tokens = parse_token_usage(stdout);
    assert_eq!(tokens, 5_678);
}

#[test]
fn parse_pr_url_multiple_in_output() {
    let stdout = "PR: https://github.com/x/y/pull/1\nAlso: https://github.com/a/b/pull/2";
    let url = parse_pr_url(stdout);
    assert_eq!(url, Some("https://github.com/x/y/pull/1".to_owned()));
}

#[test]
fn parse_pr_url_no_match_empty() {
    let url = parse_pr_url("no github links here");
    assert_eq!(url, None);
}

#[test]
fn parse_token_usage_zero_tokens() {
    let stdout = "Tokens used: 0";
    let tokens = parse_token_usage(stdout);
    assert_eq!(tokens, 0);
}

#[test]
fn parse_token_usage_error_output() {
    let stdout = "Error: something went wrong\nTokens used: 123\nExit code: 1";
    let tokens = parse_token_usage(stdout);
    assert_eq!(tokens, 123);
}

#[tokio::test]
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
                msg.contains("docker") || msg.contains("timed out"),
                "expected install/timeout error, got: {msg}"
            );
        }
    }
}

#[tokio::test]
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
    let _ = harness
        .spawn("test", &workdir, &secrets, &limits, "", agents_content)
        .await;

    let agents_path = workdir.join("AGENTS.md");
    let contents =
        std::fs::read_to_string(&agents_path).expect("AGENTS.md should have been written");
    let _ = std::fs::remove_dir_all(&workdir);

    assert_eq!(contents, agents_content);
}

#[tokio::test]
async fn kata_harness_skips_agents_md_when_empty() {
    let harness = KataHarness::with_image("nonexistent-image:latest".to_owned());
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-kata-no-agents");
    let _ = std::fs::create_dir_all(&workdir);

    let _ = harness
        .spawn("test", &workdir, &secrets, &limits, "", "")
        .await;

    let agents_path = workdir.join("AGENTS.md");
    let exists = agents_path.exists();
    let _ = std::fs::remove_dir_all(&workdir);

    assert!(
        !exists,
        "AGENTS.md should not be created when agents_md is empty"
    );
}
