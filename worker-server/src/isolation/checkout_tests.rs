use std::collections::HashMap;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::isolation::checkout;

#[test]
fn parse_github_pr_number_accepts_canonical_url() {
    let result =
        checkout::parse_github_pr_number("https://github.com/acme/widgets/pull/42?foo=bar#files");
    assert_eq!(result, Some(42));
}

#[test]
fn parse_github_pr_number_rejects_non_pr_url() {
    let result = checkout::parse_github_pr_number("https://github.com/acme/widgets/issues/42");
    assert_eq!(result, None);
}

#[test]
fn parse_github_pr_number_rejects_malformed_urls() {
    let cases = [
        "https://gitlab.com/acme/widgets/pull/42",
        "https://github.com/acme/widgets/pull/not-a-number",
        "https://github.com/acme/widgets/pull/0",
        "https://github.com/acme/widgets/pull/42/files",
        "https://github.com/acme/widgets",
    ];

    for pr_url in cases {
        assert_eq!(
            checkout::parse_github_pr_number(pr_url),
            None,
            "{pr_url} should not parse as a GitHub pull request"
        );
    }
}

#[tokio::test]
async fn checkout_pull_request_rejects_cross_repo_url() {
    let repos = vec![WorkspaceRepo {
        full_name: "acme/widgets".to_owned(),
        url: "https://github.com/acme/widgets.git".to_owned(),
        relative_path: ".".to_owned(),
    }];

    let err = checkout::checkout_pull_request(
        std::path::Path::new("/workspace"),
        &repos,
        "acme/widgets",
        "https://github.com/other/widgets/pull/42",
        &HashMap::new(),
    )
    .await
    .expect_err("cross-repo pull request URL should be rejected");

    match err {
        HarnessError::Install(message) => {
            assert_eq!(
                message,
                "pull request URL https://github.com/other/widgets/pull/42 does not belong to repo acme/widgets"
            );
        }
        HarnessError::Timeout(_)
        | HarnessError::Crash(_)
        | HarnessError::OutputParse(_)
        | HarnessError::ServerLaunch(_)
        | HarnessError::ServerUnhealthy(_)
        | HarnessError::StallDetected(_)
        | HarnessError::CancelFailed(_)
        | HarnessError::Http(_) => panic!("cross-repo URL should fail validation"),
    }
}

#[tokio::test]
async fn checkout_pull_request_rejects_malformed_github_pr_url() {
    let repos = vec![WorkspaceRepo {
        full_name: "acme/widgets".to_owned(),
        url: "https://github.com/acme/widgets.git".to_owned(),
        relative_path: ".".to_owned(),
    }];

    let err = checkout::checkout_pull_request(
        std::path::Path::new("/workspace"),
        &repos,
        "acme/widgets",
        "https://github.com/acme/widgets/pull/not-a-number",
        &HashMap::new(),
    )
    .await
    .expect_err("malformed pull request URL should be rejected");

    match err {
        HarnessError::Install(message) => {
            assert_eq!(
                message,
                "invalid GitHub pull request URL: https://github.com/acme/widgets/pull/not-a-number"
            );
        }
        HarnessError::Timeout(_)
        | HarnessError::Crash(_)
        | HarnessError::OutputParse(_)
        | HarnessError::ServerLaunch(_)
        | HarnessError::ServerUnhealthy(_)
        | HarnessError::StallDetected(_)
        | HarnessError::CancelFailed(_)
        | HarnessError::Http(_) => panic!("malformed URL should fail validation"),
    }
}

#[test]
fn checkout_branch_name_is_stable() {
    let result = checkout::checkout_branch_name(42);
    assert_eq!(result, "vulcanum-pr-42");
}
