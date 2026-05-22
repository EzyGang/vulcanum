use crate::harness::parse::{parse_pr_url, parse_token_usage};

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
    let url = parse_pr_url("no github links here");
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

#[test]
fn parse_token_usage_error_output() {
    let text = "Error: something went wrong\nTokens used: 123\nExit code: 1";
    let tokens = parse_token_usage(text);
    assert_eq!(tokens, 123);
}
