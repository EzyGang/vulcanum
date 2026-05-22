use crate::harness::host::HostHarness;
use crate::harness::parse::{parse_pr_url, parse_token_usage};
use crate::harness::{AgentHarness, ResourceLimits};

#[test]
fn parse_pr_url_valid() {
    let text = "some output\nhttps://github.com/owner/repo/pull/42\nmore output";
    let url = parse_pr_url(text);
    assert_eq!(
        url,
        Some("https://github.com/owner/repo/pull/42".to_owned())
    );
}

#[test]
fn parse_pr_url_no_match() {
    let text = "some output\nno pr url here\nmore output";
    let url = parse_pr_url(text);
    assert_eq!(url, None);
}

#[test]
fn parse_pr_url_edge_case_trailing_chars() {
    let text = "done: https://github.com/owner/repo/pull/7.";
    let url = parse_pr_url(text);
    assert_eq!(url, Some("https://github.com/owner/repo/pull/7".to_owned()));
}

#[test]
fn parse_pr_url_first_of_multiple() {
    let text = "first: https://github.com/a/b/pull/1\nsecond: https://github.com/c/d/pull/2";
    let url = parse_pr_url(text);
    assert_eq!(url, Some("https://github.com/a/b/pull/1".to_owned()));
}

#[test]
fn parse_pr_url_no_protocol() {
    let text = "output\ngithub.com/owner/repo/pull/42\nmore";
    let url = parse_pr_url(text);
    assert_eq!(url, None);
}

#[test]
fn parse_pr_url_empty_output() {
    let url = parse_pr_url("");
    assert_eq!(url, None);
}

#[test]
fn parse_token_usage_valid() {
    let text = "Tokens used: 1234\nDone";
    let tokens = parse_token_usage(text);
    assert_eq!(tokens, 1_234);
}

#[test]
fn parse_token_usage_no_match() {
    let text = "no metrics here";
    let tokens = parse_token_usage(text);
    assert_eq!(tokens, 0);
}

#[test]
fn parse_token_usage_zero() {
    let text = "Tokens used: 0";
    let tokens = parse_token_usage(text);
    assert_eq!(tokens, 0);
}

#[test]
fn parse_token_usage_large_number() {
    let text = "Tokens used: 9999999\nDone";
    let tokens = parse_token_usage(text);
    assert_eq!(tokens, 9_999_999);
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

    let result = harness
        .spawn("hello", &workdir, &secrets, &limits, "", "")
        .await;

    assert!(
        result.is_err(),
        "expected error (timeout or missing/invalid opencode)"
    );
}

#[tokio::test]
async fn host_harness_writes_agents_md() {
    let harness = HostHarness::new();
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-agents");
    let _ = std::fs::create_dir_all(&workdir);

    let agents_content = "# Test AGENTS.md\nThis is a test.";
    let _ = harness
        .spawn("test", &workdir, &secrets, &limits, "", agents_content)
        .await;

    let agents_path = workdir.join("AGENTS.md");
    let contents =
        std::fs::read_to_string(&agents_path).expect("AGENTS.md should have been written");
    let _ = std::fs::remove_dir_all(&workdir);

    assert_eq!(contents, agents_content);
}
