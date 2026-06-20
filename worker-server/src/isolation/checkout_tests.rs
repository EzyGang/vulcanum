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
fn checkout_branch_name_is_stable() {
    let result = checkout::checkout_branch_name(42);
    assert_eq!(result, "vulcanum-pr-42");
}
