use crate::util::github::{parse_github_pr_url, GithubPullRequest};

#[test]
fn parses_github_pr_url() {
    let pr = parse_github_pr_url("https://github.com/acme/widgets/pull/42")
        .expect("valid PR URL should parse");

    assert_pr(
        &pr,
        "acme/widgets",
        42,
        "https://github.com/acme/widgets/pull/42",
    );
}

#[test]
fn normalizes_github_pr_url_query_and_fragment() {
    let pr = parse_github_pr_url("https://github.com/acme/widgets/pull/42?foo=bar#files")
        .expect("valid PR URL should parse");

    assert_pr(
        &pr,
        "acme/widgets",
        42,
        "https://github.com/acme/widgets/pull/42",
    );
}

#[test]
fn rejects_non_pr_url() {
    assert!(parse_github_pr_url("https://github.com/acme/widgets/issues/42").is_none());
}

fn assert_pr(pr: &GithubPullRequest, full_name: &str, number: i64, url: &str) {
    assert_eq!(pr.repo().full_name(), full_name);
    assert_eq!(pr.number(), number);
    assert_eq!(pr.url(), url);
}
