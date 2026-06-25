use crate::services::work_runs::service::spawn_review::upsert_pr_block;

#[test]
fn upsert_pr_block_appends_block() {
    let body = "Existing task body";
    let result = upsert_pr_block(
        body,
        &["https://github.com/acme/widgets/pull/42".to_owned()],
    );

    assert!(result.starts_with(body));
    assert!(result.contains("<!-- vulcanum:prs:start -->"));
    assert!(result.contains("- https://github.com/acme/widgets/pull/42"));
    assert!(result.contains("<!-- vulcanum:prs:end -->"));
}

#[test]
fn upsert_pr_block_replaces_existing_block() {
    let body = "Before\n<!-- vulcanum:prs:start -->\nold\n<!-- vulcanum:prs:end -->\nAfter";
    let result = upsert_pr_block(
        body,
        &["https://github.com/acme/widgets/pull/43".to_owned()],
    );

    assert!(result.contains("Before"));
    assert!(result.contains("After"));
    assert!(!result.contains("old"));
    assert!(result.contains("- https://github.com/acme/widgets/pull/43"));
}
