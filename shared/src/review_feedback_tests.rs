use crate::review_feedback::review_requires_implementation;

#[test]
fn review_requires_implementation_for_critical_items() {
    let body =
        "## CRITICAL\n- Data loss on retry\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- Rename helper";

    assert!(review_requires_implementation(body));
}

#[test]
fn review_requires_implementation_for_warning_items() {
    let body = "## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None";

    assert!(review_requires_implementation(body));
}

#[test]
fn review_does_not_require_implementation_for_suggestions_only() {
    let body =
        "## CRITICAL\n- None\n\n## WARNINGS\n- No warnings\n\n## SUGGESTIONS\n- Add a helper later";

    assert!(!review_requires_implementation(body));
}
