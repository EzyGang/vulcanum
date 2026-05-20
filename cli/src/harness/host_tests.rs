use crate::harness::host::HostHarness;
use crate::harness::{AgentHarness, ResourceLimits};

#[test]
fn parse_pr_url_valid() {
    let text = "some output\nhttps://github.com/owner/repo/pull/42\nmore output";
    let url = crate::harness::host::parse_pr_url(text);
    assert_eq!(
        url,
        Some("https://github.com/owner/repo/pull/42".to_owned())
    );
}

#[test]
fn parse_pr_url_no_match() {
    let text = "some output\nno pr url here\nmore output";
    let url = crate::harness::host::parse_pr_url(text);
    assert_eq!(url, None);
}

#[test]
fn parse_pr_url_edge_case_trailing_chars() {
    let text = "done: https://github.com/owner/repo/pull/7.";
    let url = crate::harness::host::parse_pr_url(text);
    assert_eq!(url, Some("https://github.com/owner/repo/pull/7".to_owned()));
}

#[test]
fn parse_token_usage_valid() {
    let text = "Tokens used: 1234\nDone";
    let tokens = crate::harness::host::parse_token_usage(text);
    assert_eq!(tokens, 1_234);
}

#[test]
fn parse_token_usage_no_match() {
    let text = "no metrics here";
    let tokens = crate::harness::host::parse_token_usage(text);
    assert_eq!(tokens, 0);
}

#[tokio::test]
async fn host_harness_timeout_or_error() {
    let harness = HostHarness::new();
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = std::collections::HashMap::new();
    let workdir = std::env::temp_dir();

    let result = harness.spawn("hello", &workdir, &secrets, &limits).await;

    assert!(
        result.is_err(),
        "expected error (timeout or missing/invalid opencode)"
    );
}
